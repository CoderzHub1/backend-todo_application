# Todo Application API - Backend

A RESTful API backend for a todo application built with **Rust** using the **Axum** web framework and **MongoDB** for data persistence.

## Tech Stack

- **Framework**: Axum 0.8.8
- **Runtime**: Tokio
- **Database**: MongoDB 3.5.0
- **Authentication**: Bcrypt for password hashing + JWT tokens
- **JWT Library**: jsonwebtoken 10.3.0
- **Language**: Rust 2024 Edition

## Server Configuration

- **Host**: 0.0.0.0
- **Port**: 5050
- **URL**: `http://localhost:5050`

## API Endpoints

### User Management

#### 1. Create User

Creates a new user account with username, email, and password.

**Endpoint**: `POST /create-user`

**Request Body**:
```json
{
  "username": "john_doe",
  "email": "john@example.com",
  "password": "securePassword123"
}
```

**Response (Success)**:
```json
{
  "status": "Success"
}
```

**Response (Error)**:
```json
{
  "status": "Error occured while adding user"
}
```

---

#### 2. Get User

Retrieves user information by email (returns censored data without password).

**Endpoint**: `GET /get-user`

**Query Parameters**:
| Parameter | Type | Description |
|-----------|------|-------------|
| `email` | string | User's email address |

**Example Request**:
```
GET /get-user?email=john@example.com
```

**Response (User Found)**:
```json
{
  "username": "john_doe",
  "email": "john@example.com"
}
```

**Response (User Not Found)**:
```json
{
  "username": "",
  "email": ""
}
```

---

#### 3. Authenticate User

Authenticates a user with email and password. Returns a JWT token on successful authentication.

**Endpoint**: `POST /auth`

**Request Body**:
```json
{
  "email": "john@example.com",
  "pass": "securePassword123"
}
```

**Response (Success)**:
```json
{
  "auth": true,
  "error": false,
  "jwt": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

**Response (Wrong Password)**:
```json
{
  "auth": false,
  "error": true,
  "jwt": null
}
```

**Response (User Not Found)**:
```json
{
  "auth": false,
  "error": true,
  "jwt": null
}
```

**Response (Error)**:
```json
{
  "auth": false,
  "error": true,
  "jwt": null
}
```

---

#### 4. Verify JWT Token

Verifies if a JWT token is valid and returns the claims if not tampered with.

**Endpoint**: `POST /jwt-auth`

**Request Body**:
```json
{
  "jwt": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

**Response (Valid Token)**:
```json
{
  "email": "john@example.com",
  "tampered": false
}
```

**Response (Invalid/Tampered Token)**:
```json
{
  "email": null,
  "tampered": true
}
```

---

### Task Management

#### 5. Add Task

Creates a new task for an authenticated user using a JWT token.

**Endpoint**: `POST /add-task`

**Request Body**:
```json
{
  "name": "Complete project documentation",
  "priority": 1,
  "auth_jwt": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

**Response (Success)**:
```json
true
```

**Response (Failure)**:
```json
false
```

**Task Structure Created** (stored in database):
```json
{
  "id": 1,
  "name": "Complete project documentation",
  "status": false,
  "priority": 1
}
```

---

#### 6. Get Tasks

Retrieves tasks for an authenticated user with pagination. The `counter` parameter specifies the maximum number of tasks to retrieve (minimum of 5).

**Endpoint**: `GET /get-tasks`

**Request Body**:
```json
{
  "auth_jwt": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "counter": 10
}
```

**Parameters**:
| Parameter | Type | Description |
|-----------|------|-------------|
| `auth_jwt` | string | Valid JWT token received from `/auth` |
| `counter` | integer | Maximum number of tasks to retrieve (minimum 5) |

**Response (Success)**:
```json
{
  "res": true,
  "tasks": [
    {
      "id": 1,
      "name": "Complete project documentation",
      "status": false,
      "priority": 1
    },
    {
      "id": 2,
      "name": "Review pull requests",
      "status": true,
      "priority": 2
    }
  ]
}
```

**Response (Failure)**:
```json
{
  "res": false,
  "tasks": null
}
```

---

#### 7. Update Task

Toggles a task's completion status (each call flips `status` between `true` and `false`).

**Endpoint**: `POST /update-task`

**Request Body**:
```json
{
  "id": 1,
  "auth_jwt": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

**Parameters**:
| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | integer | Task ID to update |
| `auth_jwt` | string | Valid JWT token received from `/auth` |

**Response (Success)**:
```json
{
  "success": true
}
```

**Response (Failure)**:
```json
{
  "success": false
}
```

**Behavior Notes**:
- If the task is currently incomplete (`status: false`), calling `/update-task` changes it to complete (`status: true`).
- If the task is currently complete (`status: true`), calling `/update-task` changes it back to incomplete (`status: false`).
- If the task ID does not exist for the user, the API returns failure.

---

#### 8. Delete Task

Deletes a task for an authenticated user.

**Endpoint**: `POST /delete-task`

**Request Body**:
```json
{
  "auth_jwt": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "task_id": 1
}
```

**Parameters**:
| Parameter | Type | Description |
|-----------|------|-------------|
| `auth_jwt` | string | Valid JWT token received from `/auth` |
| `task_id` | integer | Task ID to delete |

**Response (Success)**:
```json
{
  "success": true
}
```

**Response (Failure)**:
```json
{
  "success": false
}
```

## Data Models

### User
```rust
{
  "username": String,
  "email": String,
  "password": String  // Bcrypt hashed
}
```

### Task
```rust
{
  "id": usize,
  "name": String,
  "status": bool,
  "priority": usize
}
```

## Database Structure

### Collections
- **MongoDB Users**: `todo_userdata` database
  - Collection: `users_coll`
  
- **MongoDB Tasks**: `todo_tasks` database
  - Collection: One per user (identified by email)

## Authentication Flow

### Password-Based Authentication (Legacy)

1. User creates an account via `/create-user`
2. User authenticates via `/auth` endpoint with email and password
3. Backend verifies credentials using bcrypt password verification
4. Upon success, a JWT token is returned

### JWT-Based Authentication

1. User obtains a JWT token by authenticating via `/auth` endpoint
2. User includes the JWT token in the `auth_jwt` field for protected endpoints (e.g., `/add-task`)
3. Backend verifies JWT token signature and claims
4. User can verify JWT token validity using `/jwt-auth` endpoint

### Task Operations

- For `/add-task`: Include `auth_jwt` from the `/auth` response
- For `/delete-task`: Include `auth_jwt` from the `/auth` response
- For `/update-task`: Include `auth_jwt` from the `/auth` response
- For `/get-tasks`: Include `auth_jwt` from the `/auth` response

## Running the Server

```bash
cargo run --release
```

The server will start on `http://0.0.0.0:5050` and output:
```
Server is now running on http://localhost:5050
```

## Testing the Server

You can run an automated integration test against the backend API using the provided Python script. It verifies user creation, authentication, task operations, and task-status toggling behavior.

To automatically start the backend server and run the tests:
```bash
python3 test/api_smoke_test.py --start-server
```

If your server is already running, you can run the test script directly against it:
```bash
python3 test/api_smoke_test.py
```

## CORS Policy

The API uses a permissive CORS policy, allowing requests from any origin. Modify the `CorsLayer` configuration in `main.rs` for production deployments.

## Development Notes

- Passwords are hashed using bcrypt with a cost factor of 12 (DEFAULT_COST)
- JWT tokens are generated using HS256 algorithm with a configurable secret key (defaults to "my_jwt_secret123")
- All API endpoints return JSON responses
- Database operations use MongoDB drivers with async/await pattern
- The application uses Tokio for async runtime
- JWT tokens contain user email in the claims and can be verified without database access


`Note: this readme.md is AI generated but the the details contained about the usage of the API are checked and are completely correct`
