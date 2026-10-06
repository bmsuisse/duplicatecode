# /// script
# requires-python = ">=3.10,<3.13"
# dependencies = ["embed-anything", "numpy"]
# ///
"""Local OpenAI-compatible /v1/embeddings server around embed-anything (no API key needed).

    uv run --no-project --python 3.12 eval/embed_server.py          # Qwen/Qwen3-Embedding-0.6B on :8099
    export DUPLICATECODE_EMBED_ENDPOINT=http://127.0.0.1:8099/v1 DUPLICATECODE_EMBED_API_KEY=local
    export DUPLICATECODE_EMBED_MODEL=qwen3-embedding-0.6b
    duplicatecode scan . --embed-code

Env: EMBED_MODEL (HF id), EMBED_PORT. The `dimensions` request field is ignored (full vectors).
CPU only with the PyPI wheel; a CUDA build of embed-anything is needed to use the GPU.
"""
import json
import os
from http.server import BaseHTTPRequestHandler, HTTPServer

import embed_anything
from embed_anything import EmbeddingModel

MODEL_ID = os.environ.get("EMBED_MODEL", "Qwen/Qwen3-Embedding-0.6B")
PORT = int(os.environ.get("EMBED_PORT", "8099"))
model = EmbeddingModel.from_pretrained_hf(MODEL_ID)


class Handler(BaseHTTPRequestHandler):
    def do_POST(self) -> None:
        if not self.path.rstrip("/").endswith("/embeddings"):
            self.send_error(404)
            return
        body = json.loads(self.rfile.read(int(self.headers.get("content-length", 0))))
        texts = body["input"] if isinstance(body["input"], list) else [body["input"]]
        vecs = [r.embedding for r in embed_anything.embed_query(texts, model)]
        out = json.dumps({
            "object": "list",
            "model": MODEL_ID,
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
    print(f"serving {MODEL_ID} on http://127.0.0.1:{PORT}/v1", flush=True)
    HTTPServer(("127.0.0.1", PORT), Handler).serve_forever()
