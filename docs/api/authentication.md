# Authentication

## Register

```http
POST /auth/register
Content-Type: application/json

{"email": "user@example.com", "password": "...", "display_name": "User"}
```

## Login

```http
POST /auth/login
Content-Type: application/json

{"email": "user@example.com", "password": "..."}
```

Returns a JWT token:

```json
{"token": "eyJ...", "user": {"id": "...", "email": "...", "display_name": "..."}}
```

## Using the Token

Include the token in all subsequent requests:

```
Authorization: Bearer eyJ...
```

## Desktop Mode

In desktop mode, a local user is created automatically on first launch. The app handles authentication transparently.
