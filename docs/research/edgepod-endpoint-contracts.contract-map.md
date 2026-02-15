## Edge-Pod Compact Endpoint Contract Map
Source: `/Users/damarpanuluh/MERIDIAN-NEW/gotong-royong/docs/research/edgepod-endpoint-contracts.schema.json`

Generated for implementation handoff and endpoint-task creation.

Prompt-id pinning:
- EP-03: `DUPLICATE-001`
- EP-05: `GAMING-001`
- EP-08: `SENSITIVE-001`
- EP-09: `CREDIT-001`

---

### EP-00
- endpoint: /api/v1/edge-pod/ai/00/triage
- method: POST
- request_schema: "#/endpoints/EP-00/request_schema"
- success_response_schema: "#/endpoints/EP-00/success_response_schema"
- error_response_schema: "#/endpoints/EP-00/error_response_schema"

### EP-01
- endpoint: /api/v1/edge-pod/ai/01/classification
- method: POST
- request_schema: "#/endpoints/EP-01/request_schema"
- success_response_schema: "#/endpoints/EP-01/success_response_schema"
- error_response_schema: "#/endpoints/EP-01/error_response_schema"

### EP-02
- endpoint: /api/v1/edge-pod/ai/02/redaction
- method: POST
- request_schema: "#/endpoints/EP-02/request_schema"
- success_response_schema: "#/endpoints/EP-02/success_response_schema"
- error_response_schema: "#/endpoints/EP-02/error_response_schema"

### EP-03
- endpoint: /api/v1/edge-pod/ai/03/duplicate-detection
- method: POST
- request_schema: "#/endpoints/EP-03/request_schema"
- success_response_schema: "#/endpoints/EP-03/success_response_schema"
- error_response_schema: "#/endpoints/EP-03/error_response_schema"

### EP-04
- endpoint: /api/v1/edge-pod/ai/04/moderation
- method: POST
- request_schema: "#/endpoints/EP-04/request_schema"
- success_response_schema: "#/endpoints/EP-04/success_response_schema"
- error_response_schema: "#/endpoints/EP-04/error_response_schema"

### EP-05
- endpoint: /api/v1/edge-pod/ai/05/gaming-risk
- method: POST
- request_schema: "#/endpoints/EP-05/request_schema"
- success_response_schema: "#/endpoints/EP-05/success_response_schema"
- error_response_schema: "#/endpoints/EP-05/error_response_schema"

### EP-06
- endpoint: /api/v1/edge-pod/ai/06/criteria-suggestions
- method: POST
- request_schema: "#/endpoints/EP-06/request_schema"
- success_response_schema: "#/endpoints/EP-06/success_response_schema"
- error_response_schema: "#/endpoints/EP-06/error_response_schema"

### EP-07
- endpoint: /api/v1/edge-pod/ai/07/summary
- method: POST
- request_schema: "#/endpoints/EP-07/request_schema"
- success_response_schema: "#/endpoints/EP-07/success_response_schema"
- error_response_schema: "#/endpoints/EP-07/error_response_schema"

### EP-08
- endpoint: /api/v1/edge-pod/ai/08/sensitive-media
- method: POST
- request_schema: "#/endpoints/EP-08/request_schema"
- success_response_schema: "#/endpoints/EP-08/success_response_schema"
- error_response_schema: "#/endpoints/EP-08/error_response_schema"

### EP-09
- endpoint: /api/v1/edge-pod/ai/09/credit-recommendation
- method: POST
- request_schema: "#/endpoints/EP-09/request_schema"
- success_response_schema: "#/endpoints/EP-09/success_response_schema"
- error_response_schema: "#/endpoints/EP-09/error_response_schema"

### EP-10
- endpoint: /api/v1/edge-pod/ai/10/skill-extract
- method: POST
- request_schema: "#/endpoints/EP-10/request_schema"
- success_response_schema: "#/endpoints/EP-10/success_response_schema"
- error_response_schema: "#/endpoints/EP-10/error_response_schema"

### EP-11
- endpoint: /api/v1/edge-pod/ai/siaga/evaluate
- method: POST
- request_schema: "#/endpoints/EP-11/request_schema"
- success_response_schema: "#/endpoints/EP-11/success_response_schema"
- error_response_schema: "#/endpoints/EP-11/error_response_schema"
