# /// script
# requires-python = ">=3.10,<3.13"
# dependencies = ["embed-anything", "numpy"]
# ///
"""Local OpenAI-compatible /v1/embeddings server around embed-anything (no API key needed).

    uv run --no-project --python 3.12 eval/embed_server.py          # on :8099
    duplicatecode scan . --embed minilm

The server loads whichever Hugging Face model id the request names (`"model": ...`) the first time it is
asked for it, so `duplicatecode scan . --embed minilm|qwen3|potion` works against one running server and
cached vectors can never be mixed between models. Env: EMBED_MODEL (model for requests that name none),
EMBED_PORT. The `dimensions` request field is ignored (full vectors).
CPU only with the PyPI wheel; a CUDA build of embed-anything is needed to use the GPU.
"""
import json
import os
from http.server import BaseHTTPRequestHandler, HTTPServer

import embed_anything
from embed_anything import EmbeddingModel

DEFAULT_MODEL = os.environ.get("EMBED_MODEL", "sentence-transformers/all-MiniLM-L6-v2")
PORT = int(os.environ.get("EMBED_PORT", "8099"))
_models: dict[str, EmbeddingModel] = {}


def load(model_id: str) -> EmbeddingModel:
    if model_id not in _models:
        print(f"loading {model_id}", flush=True)
        _models[model_id] = EmbeddingModel.from_pretrained_hf(model_id)
    return _models[model_id]


class Handler(BaseHTTPRequestHandler):
    def do_POST(self) -> None:
        if not self.path.rstrip("/").endswith("/embeddings"):
            self.send_error(404)
            return
        body = json.loads(self.rfile.read(int(self.headers.get("content-length", 0))))
        texts = body["input"] if isinstance(body["input"], list) else [body["input"]]
        model_id = body.get("model") or DEFAULT_MODEL
        try:
            model = load(model_id)
        except BaseException as e:  # the native loader panics on unsupported architectures
            err = json.dumps({"error": f"cannot load {model_id}: {e}"[:300]}).encode()
            self.send_response(400)
            self.send_header("content-length", str(len(err)))
            self.end_headers()
            self.wfile.write(err)
            return
        vecs = [r.embedding for r in embed_anything.embed_query(texts, model)]
        out = json.dumps({
            "object": "list",
            "model": model_id,
            "data": [{"object": "embedding", "index": i, "embedding": v} for i, v in enumerate(vecs)],
        }).encode()
        self.send_response(200)
        self.send_header("content-type", "application/json")
        self.send_header("content-length", str(len(out)))
        self.end_headers()
        self.wfile.write(out)

    def log_message(self, *a) -> None:  # quiet
        pass


if __name__ == "__main__":
    print(f"serving on http://127.0.0.1:{PORT}/v1 (default model {DEFAULT_MODEL})", flush=True)
    HTTPServer(("127.0.0.1", PORT), Handler).serve_forever()
