# Chat Attachment S3 Smoke Report

Date: 2026-02-25T20:16:43Z
Mode: `live`
API URL: `http://127.0.0.1:50062`
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

- Attachment ID: `019c9673ece27632bcd2c8a785096cf6`
- Signed download path: `/v1/chat/attachments/019c9673ece27632bcd2c8a785096cf6/download?exp=1772137105634&sig=928be603fff7f8e49aefdca8d5660bdd70d8ad54b6f26f6f1ce237950456e6c9`
- Presigned object URL host: `127.0.0.1:9000`
