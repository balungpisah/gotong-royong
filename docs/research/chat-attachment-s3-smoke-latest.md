# Chat Attachment S3 Smoke Report

Date: 2026-02-25T18:04:41Z
Mode: `live`
API URL: `http://127.0.0.1:53179`
S3 Endpoint: `http://127.0.0.1:9000`
S3 Bucket: `gotong-royong-evidence-dev`
Attachment Backend: `s3`

## Checks

| Check | Status |
|---|---|
| Upload `POST /v1/chat/attachments/upload` | PASS |
| Download gate `GET /v1/chat/attachments/:id/download` returns 307 | PASS |
| Presigned object URL fetch from MinIO returns 200 | PASS |
| Uploaded bytes match downloaded bytes | PASS |

## Sample Artifact

- Attachment ID: `019c95fa07a472d18c1a88e3b5e78792`
- Signed download path: `/v1/chat/attachments/019c95fa07a472d18c1a88e3b5e78792/download?exp=1772129117092&sig=d4af770eee7fc25bd1c8cee4746bcdf0fd27824119d8f632bfdd2b9d1110f4ed`
- Presigned object URL host: `127.0.0.1:9000`
