# Project Plan & Context

This document summarizes the architecture, quirks, and next steps for the HermesAI monorepo.

## Monorepo Structure
- **apps/dropcode-client**: Angular 20 frontend (standalone components, Bootstrap theme).
- **apps/execution-sandbox**: Python Flask backend + Nginx for static/UI serving and API proxy.
- **apps/database**: PostgreSQL container providing durable state storage.
- **/.canvas**: Special directory for live testing standalone HTML/JS/CSS snippets.

### Frontend (dropcode-client)
- **Angular 20** using standalone components.
- Auth flow:
  - Login / Register components (Bootstrap forms).
  - ThirdPartyAuthComponent: GitHub login (can extend to Google/Facebook).
  - AuthService: manages currentUser$ via cookie-based JWT.
  - Guards: RxJS-based, wait until auth state is resolved before redirecting.
- Routing:
  - `/login`, `/register` (public).
  - `/` → HomeComponent (requires auth).
  - `/features`, `/pricing` → placeholder pages (requires auth).
  - `/canvas` → CanvasComponent (requires auth, shows live preview).
- Tests:
  - Use `ng test` (Karma + Jasmine). Jest is **not supported** in Angular 20.

### Backend (execution-sandbox)
- **Flask + Flask-JWT-Extended**.
- **PostgreSQL** database managed in `apps/database`.
- **Flask-SQLAlchemy** + **Flask-Migrate** for ORM and schema migrations.
- SocketIO for workspace events.
- Auth:
  - Local login/register endpoints.
  - GitHub OAuth implemented with `/auth/redirect/github` and `/auth/callback/github`.
  - Tokens issued as `auth_token` cookies (HttpOnly, Secure, SameSite=Lax).
  - `/auth/me` returns current user from cookie.
  - `/auth/logout` clears cookie.
- Routes:
  - `/api/canvas` → serves `/.canvas/index.html` (if present).
  - `/api/canvas/assets/*` → serves supporting static files from `/.canvas/assets/`.
  - `/api/waitlist` → manages early-access signups.
- Nginx:
  - Serves Angular app under `/`.
  - Proxies `/api/*` → Flask.
  - Config includes `try_files $uri /index.html;` to support Angular 20 path routing.

---

## Database Architecture (PostgreSQL)

### Overview
A dedicated **PostgreSQL database container** lives under `apps/database`, serving as the main persistence layer for authentication, user workspaces, and the waitlist MVP. This modular architecture allows scaling and durability when transitioning to Kubernetes.

### Directory Structure
```
apps/database/
├── Dockerfile
└── init.sql (optional, bootstrap schema)
```

### Dockerfile (apps/database/Dockerfile)
```dockerfile
FROM postgres:15-alpine
ENV POSTGRES_USER=postgres \
    POSTGRES_PASSWORD=postgres \
    POSTGRES_DB=dropcode
EXPOSE 5432
VOLUME ["/var/lib/postgresql/data"]
COPY init.sql /docker-entrypoint-initdb.d/
```

### Flask ORM Integration

Dependencies (in `apps/execution-sandbox/requirements.txt`):
```
flask-sqlalchemy
flask-migrate
psycopg2-binary
```

#### Configuration (`apps/execution-sandbox/sandbox_server/__init__.py`)
```python
from flask import Flask
from flask_sqlalchemy import SQLAlchemy
from flask_migrate import Migrate
import os

db = SQLAlchemy()
migrate = Migrate()

def create_app():
    app = Flask(__name__)
    app.config['SQLALCHEMY_DATABASE_URI'] = os.getenv(
        'DATABASE_URL',
        'postgresql+psycopg2://postgres:postgres@db:5432/dropcode'
    )
    app.config['SQLALCHEMY_TRACK_MODIFICATIONS'] = False

    db.init_app(app)
    migrate.init_app(app, db)

    from .routes import register_routes
    register_routes(app)

    return app
```

#### Models (`apps/execution-sandbox/sandbox_server/models.py`)
```python
from datetime import datetime
from . import db

class User(db.Model):
    id = db.Column(db.Integer, primary_key=True)
    email = db.Column(db.String(120), unique=True, nullable=False)
    password_hash = db.Column(db.String(255), nullable=False)
    created_at = db.Column(db.DateTime, default=datetime.utcnow)

    workspaces = db.relationship('Workspace', backref='user', lazy=True)

class Workspace(db.Model):
    id = db.Column(db.Integer, primary_key=True)
    user_id = db.Column(db.Integer, db.ForeignKey('user.id'), nullable=False)
    container_url = db.Column(db.String(255))
    created_at = db.Column(db.DateTime, default=datetime.utcnow)

class Waitlist(db.Model):
    id = db.Column(db.Integer, primary_key=True)
    email = db.Column(db.String(120), unique=True, nullable=False)
    created_at = db.Column(db.DateTime, default=datetime.utcnow)
    source = db.Column(db.String(100))
    notes = db.Column(db.Text)
```

#### Waitlist Route (`apps/execution-sandbox/sandbox_server/routes/waitlist.py`)
```python
from flask import Blueprint, request, jsonify
from datetime import datetime
from ..models import db, Waitlist

bp = Blueprint('waitlist', __name__, url_prefix='/api')

@bp.route('/waitlist', methods=['POST'])
def join_waitlist():
    data = request.get_json()
    email = data.get('email')
    if not email:
        return jsonify({'error': 'Email required'}), 400

    existing = Waitlist.query.filter_by(email=email).first()
    if existing:
        return jsonify({'message': "You're already on the list!"}), 409

    entry = Waitlist(email=email, created_at=datetime.utcnow())
    db.session.add(entry)
    db.session.commit()

    return jsonify({'message': 'Welcome to the waitlist!'}), 201
```

#### Routes Registration (`apps/execution-sandbox/sandbox_server/routes/__init__.py`)
```python
def register_routes(app):
    from .waitlist import bp as waitlist_bp
    app.register_blueprint(waitlist_bp)
```

---

### Migration Commands
Run inside the Flask container:
```bash
flask db init
flask db migrate -m "initial schema"
flask db upgrade
```

---

### Kubernetes-Ready Design
- PostgreSQL runs as a **StatefulSet** with persistent storage.
- Flask connects via Service DNS: `postgresql+psycopg2://postgres:postgres@postgres-svc:5432/dropcode`.
- Horizontal scaling affects only Flask replicas, not the DB.

---

### Summary
This integration provides:
- Durable PostgreSQL database container.
- Idiomatic Flask ORM with SQLAlchemy + Alembic migrations.
- Extensible schema for users, workspaces, and waitlist entries.
- Clean upgrade path to Kubernetes with persistent storage and service-based networking.
