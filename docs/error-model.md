# Error Model

All API errors return:

```json
{
  "error": {
    "code": "FORBIDDEN",
    "message": "cannot mutate another merchant's offer"
  }
}
```

## Error codes

- `UNAUTHORIZED` -> `401`
- `FORBIDDEN` -> `403`
- `NOT_FOUND` -> `404`
- `BAD_REQUEST` -> `400`
- `INTERNAL_SERVER_ERROR` -> `500`

## Mapping principles

- Authentication failures return `UNAUTHORIZED`.
- Role/ownership violations return `FORBIDDEN`.
- Identity-link conflicts (same username already bound to a different `sub`) return `FORBIDDEN`.
- Missing resources return `NOT_FOUND`.
- Input validation failures return `BAD_REQUEST` (malformed UUID, invalid enum-like values, invalid date format).
- Unexpected failures return `INTERNAL_SERVER_ERROR`.

