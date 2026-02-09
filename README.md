# Todo Application API - Backend

A RESTful API backend for a todo application built with **Rust** using the **Axum** web framework and **MongoDB** for data persistence.

## Tech Stack

- **Framework**: Axum 0.8.8
- **Runtime**: Tokio
- **Database**: MongoDB 3.5.0
- **Authentication**: Bcrypt for password hashing
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

Authenticates a user with email and password.

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
  "error": false
}
```

**Response (Wrong Password)**:
```json
{
  "auth": false,
  "error": false
}
```

**Response (User Not Found)**:
```json
{
  "auth": false,
  "error": false
}
```

**Response (Error)**:
```json
{
  "auth": false,
  "error": true
}
```

---

### Task Management

#### 4. Add Task

Creates a new task for an authenticated user.

**Endpoint**: `POST /add-task`

**Request Body**:
```json
{
  "name": "Complete project documentation",
  "priority": 1,
  "email": "john@example.com",
  "auth_pass": "securePassword123"
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

#### 5. Get Tasks

Retrieves tasks for an authenticated user with pagination.

**Endpoint**: `GET /get-tasks`

**Request Body**:
```json
{
  "email": "john@example.com",
  "counter": 10
}
```

**Parameters**:
| Parameter | Type | Description |
|-----------|------|-------------|
| `email` | string | User's email address |
| `counter` | integer | Maximum number of tasks to retrieve |

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

#### 6. Update Task

Marks a task as complete (updates status to true).

**Endpoint**: `POST /update-task`

**Request Body**:
```json
{
  "id": 1,
  "email": "john@example.com"
}
```

**Parameters**:
| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | integer | Task ID to update |
| `email` | string | User's email address |

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

---

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

## Error Handling

The API provides error responses with descriptive messages:
- **400 Bad Request**: Invalid request body or missing required fields
- **500 Internal Server Error**: Database or server-side errors

Errors are logged to stderr with detailed information for debugging.

## Authentication Flow

1. User creates an account via `/create-user`
2. User authenticates via `/auth` endpoint
3. User includes `auth_pass` field in requests that require authentication (add-task, update-task, get-tasks)
4. Backend verifies credentials using bcrypt password verification

## Running the Server

```bash
cargo run --release
```

The server will start on `http://0.0.0.0:5050` and output:
```
Server is now running on http://localhost:5050
```

## CORS Policy

The API uses a permissive CORS policy, allowing requests from any origin. Modify the `CorsLayer` configuration in `main.rs` for production deployments.

## Development Notes

- Passwords are hashed using bcrypt with a cost factor of 12 (DEFAULT_COST)
- All API endpoints return JSON responses
- Database operations use MongoDB drivers with async/await pattern
- The application uses Tokio for async runtime


`Note: this readme.md is AI generated but the the details contained are correct`