# /// script
# requires-python = ">=3.10,<3.13"
# dependencies = ["embed-anything==0.7.1", "numpy==2.5.3"]
# ///
"""Local OpenAI-compatible /v1/embeddings server around embed-anything (no API key needed).

    uv run --no-project --python 3.12 eval/embed_server.py          # on :8099
    duplicatecode scan . --embed minilm

The server loads the Hugging Face model id the request names (`"model": ...`) the first time it is
asked for it, so `duplicatecode scan . --embed minilm|qwen3|potion` works against one running server and
cached vectors can never be mixed between models. Only allowlisted ids load: the presets
sentence-transformers/all-MiniLM-L6-v2, Qwen/Qwen3-Embedding-0.6B, minishlab/potion-base-8M, EMBED_MODEL, and any
ids in the comma-separated env var EMBED_MODELS. Requests must send `Content-Type: application/json` (forces a
CORS preflight, which this server never answers, so web pages cannot use it), a loopback `Host` header, and a
body of at most 8 MB. Errors are answered as JSON `{"error": ...}` with HTTP 400/500.
Env: EMBED_MODEL (model for requests that name none), EMBED_MODELS, EMBED_PORT.
The `dimensions` request field is ignored (full vectors).
CPU only with the PyPI wheel; a CUDA build of embed-anything is needed to use the GPU.
"""
import json
import os
from http.server import BaseHTTPRequestHandler, HTTPServer

import embed_anything
from embed_anything import EmbeddingModel

DEFAULT_MODEL = os.environ.get("EMBED_MODEL", "sentence-transformers/all-MiniLM-L6-v2")
PORT = int(os.environ.get("EMBED_PORT", "8099"))
PRESETS = {"sentence-transformers/all-MiniLM-L6-v2", "Qwen/Qwen3-Embedding-0.6B", "minishlab/potion-base-8M"}
ALLOWED = PRESETS | {DEFAULT_MODEL} | {m.strip() for m in os.environ.get("EMBED_MODELS", "").split(",") if m.strip()}
MAX_BODY = 8 * 1024 * 1024
LOOPBACK = {"127.0.0.1", "localhost", "[::1]"}
_models: dict[str, EmbeddingModel] = {}


class BadRequest(Exception):
    pass


def load(model_id: str) -> EmbeddingModel:
    if model_id not in ALLOWED:
        raise BadRequest(f"model {model_id!r} is not allowed; set EMBED_MODELS to add it")
    if model_id not in _models:
        print(f"loading {model_id}", flush=True)
        try:
            _models[model_id] = EmbeddingModel.from_pretrained_hf(model_id)
        except BaseException as e:  # the native loader panics (BaseException) on unsupported architectures
            if isinstance(e, (KeyboardInterrupt, SystemExit)):
                raise
            raise BadRequest(f"cannot load {model_id}: {e}") from e
    return _models[model_id]


class Handler(BaseHTTPRequestHandler):
    def reply(self, code: int, obj: dict) -> None:
        out = json.dumps(obj).encode()
        self.send_response(code)
        self.send_header("content-type", "application/json")
        self.send_header("content-length", str(len(out)))
        self.end_headers()
        self.wfile.write(out)

    def handle_embeddings(self) -> dict:
        host = (self.headers.get("host") or "").lower()
        host = host[:host.index("]") + 1] if host.startswith("[") else host.split(":")[0]
        if host not in LOOPBACK:
            raise BadRequest("non-loopback Host header")
        if not (self.headers.get("content-type") or "").split(";")[0].strip().lower() == "application/json":
            raise BadRequest("Content-Type must be application/json")
        try:
            length = int(self.headers.get("content-length", ""))
        except ValueError:
            raise BadRequest("missing or invalid Content-Length") from None
        if length < 0 or length > MAX_BODY:
            raise BadRequest(f"body must be 0..{MAX_BODY} bytes")
        try:
            body = json.loads(self.rfile.read(length))
        except ValueError as e:
            raise BadRequest(f"invalid JSON: {e}") from e
        if not isinstance(body, dict) or "input" not in body:
            raise BadRequest("missing `input`")
        texts = body["input"] if isinstance(body["input"], list) else [body["input"]]
        if not all(isinstance(t, str) for t in texts):
            raise BadRequest("`input` must be a string or a list of strings")
        model_id = body.get("model") or DEFAULT_MODEL
        model = load(model_id)
        vecs = [r.embedding for r in embed_anything.embed_query(texts, model)]
        return {
            "object": "list",
            "model": model_id,
            "data": [{"object": "embedding", "index": i, "embedding": v} for i, v in enumerate(vecs)],
        }

    def do_POST(self) -> None:
        if not self.path.rstrip("/").endswith("/embeddings"):
            self.send_error(404)
            return
        try:
            self.reply(200, self.handle_embeddings())
        except BadRequest as e:
            self.reply(400, {"error": str(e)[:300]})
        except Exception as e:  # noqa: BLE001
            self.reply(500, {"error": f"{type(e).__name__}: {e}"[:300]})

    def log_message(self, *a) -> None:  # quiet
        pass


if __name__ == "__main__":
    print(f"serving on http://127.0.0.1:{PORT}/v1 (default model {DEFAULT_MODEL})", flush=True)
    HTTPServer(("127.0.0.1", PORT), Handler).serve_forever()
