# Kubernetes Plan

This document outlines the steps to transform the `docker-compose.yml` configuration into Kubernetes manifests. Each service defined in the Docker Compose file will become a Kubernetes Deployment and corresponding Service. We'll also configure volumes, networks, and environment appropriately.

## Prerequisites

- A Kubernetes cluster up and running
- `kubectl` configured to interact with the cluster
- `helm` (optional) for managing Kubernetes applications

## Components

### 1. Volumes

#### Persistent Volume Claims (PVC)
- Create PVC for `db_data`:

```yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: db-data-pvc
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 1Gi
```

### 2. ConfigMaps & Secrets

#### Secrets
- Create a Secret for the database credentials:

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: db-credentials
type: Opaque
data:
  USERNAME: aGVybWVzYWlfdXNlcg== # base64 encoded 'hermesai_user'
  PASSWORD: c2VjcmV0           # base64 encoded 'secret'
```

#### ConfigMaps
- Use ConfigMaps for environment variables or config files like `nginx.conf`.

### 3. Deployments

Each service from the Docker Compose configuration will have a corresponding Deployment.

#### Database Deployment
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: hermesai-db
spec:
  replicas: 1
  selector:
    matchLabels:
      app: hermesai-db
  template:
    metadata:
      labels:
        app: hermesai-db
    spec:
      containers:
      - name: hermesai-db
        image: your-registry/hermesai-db:latest
        env:
        - name: POSTGRES_DB
          value: hermesaidb
        - name: POSTGRES_USER
          valueFrom:
            secretKeyRef:
              name: db-credentials
              key: USERNAME
        - name: POSTGRES_PASSWORD
          valueFrom:
            secretKeyRef:
              name: db-credentials
              key: PASSWORD
        volumeMounts:
        - mountPath: /var/lib/postgresql/data
          name: db-data-volume
        ports:
        - containerPort: 5432
      volumes:
      - name: db-data-volume
        persistentVolumeClaim:
          claimName: db-data-pvc
```

#### Backend Deployment
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: hermesai-backend
spec:
  replicas: 1
  selector:
    matchLabels:
      app: hermesai-backend
  template:
    metadata:
      labels:
        app: hermesai-backend
    spec:
      containers:
      - name: hermesai-backend
        image: your-registry/hermesai-backend:latest
        env:
        - name: DATABASE_URL
          value: postgres://hermesai_user:secret@hermesai-db:5432/hermesaidb
        - name: PYTHONUNBUFFERED
          value: "1"
        ports:
        - containerPort: 8080
```

#### Frontend Deployment
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: hermesai-frontend
spec:
  replicas: 1
  selector:
    matchLabels:
      app: hermesai-frontend
  template:
    metadata:
      labels:
        app: hermesai-frontend
    spec:
      containers:
      - name: hermesai-frontend
        image: your-registry/hermesai-frontend:latest
        env:
        - name: NODE_ENV
          value: production
        ports:
        - containerPort: 80
```

#### Router Deployment
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: hermesai-router
spec:
  replicas: 1
  selector:
    matchLabels:
      app: hermesai-router
  template:
    metadata:
      labels:
        app: hermesai-router
    spec:
      containers:
      - name: hermesai-router
        image: your-registry/hermesai-router:latest
        ports:
        - containerPort: 80
        - containerPort: 443
        env:
        - name: ROUTER_MODE
          value: production
        volumeMounts:
        - name: nginx-config-volume
          mountPath: /etc/nginx/nginx.conf
          subPath: nginx.conf
        args:
        - "curl"
        - "-f"
        - "http://localhost/api/healthcheck"
```

#### Tunnel Deployment
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: hermesai-tunnel
spec:
  replicas: 1
  selector:
    matchLabels:
      app: hermesai-tunnel
  template:
    metadata:
      labels:
        app: hermesai-tunnel
    spec:
      containers:
      - name: hermesai-tunnel
        image: your-registry/hermesai-tunnel:latest
        volumeMounts:
        - name: cloudflared-config
          mountPath: /etc/cloudflared
      volumes:
      - name: cloudflared-config
        hostPath:
          path: /etc/cloudflared
          type: DirectoryOrCreate
```

### 4. Services

#### Database Service
```yaml
apiVersion: v1
kind: Service
metadata:
  name: db-service
spec:
  selector:
    app: hermesai-db
  ports:
    - protocol: TCP
      port: 5432
      targetPort: 5432
  type: ClusterIP
```

#### Backend Service
```yaml
apiVersion: v1
kind: Service
metadata:
  name: backend-service
spec:
  selector:
    app: hermesai-backend
  ports:
    - protocol: TCP
      port: 8080
      targetPort: 8080
  type: ClusterIP
```

#### Frontend and Router Services
```yaml
apiVersion: v1
kind: Service
metadata:
  name: frontend-service
spec:
  selector:
    app: hermesai-frontend
  ports:
    - protocol: TCP
      port: 80
      targetPort: 80
  type: ClusterIP
```

```yaml
apiVersion: v1
kind: Service
metadata:
  name: router-service
spec:
  selector:
    app: hermesai-router
  ports:
    - protocol: TCP
      port: 80
      targetPort: 80
    - protocol: TCP
      port: 443
      targetPort: 443
  type: LoadBalancer
```

### 5. Networking

- Network Policies can be configured to control traffic between services if necessary.
- Use a Service Mesh for enhanced traffic management, observability, and security.

## Additional Considerations

- Ensure images (`your-registry/your-service:latest`) are correctly built and available.
- Consider autoscaling configurations (`HorizontalPodAutoscaler`) for managing load.
- Deploy Ingress Controllers for routing external traffic.
- Monitor and log using compatible Kubernetes tools (Prometheus, Grafana, etc.).

By following this plan, you can transition your Docker Compose setup to a Kubernetes environment, preserving the configuration and logic defined in the original `docker-compose.yml` file.


------ FILE LEVEL SUMMARY ------
# Summary of Required Changes for Kubernetes Migration

This summary outlines the essential changes needed to migrate the `docker-compose.yml` configuration to Kubernetes.

## Files and Required Changes

### 1. Persistent Volume Claims
- **File**: `persistent-volume-claims.yaml`
    - Define PVC for `db_data`.

### 2. Secrets
- **File**: `secrets.yaml`
    - Create a Secret for the database credentials.

### 3. ConfigMaps
- **File**: `configmaps.yaml`
    - Use ConfigMaps for environment variables and configuration files (e.g., `nginx.conf`).

### 4. Deployments

- **File**: `database-deployment.yaml`
    - Define the Deployment for `hermesai-db`.
    - Use Secret for sensitive data.

- **File**: `backend-deployment.yaml`
    - Define the Deployment for `hermesai-backend`.
    - Connect to the database using a Secret-derived URL.

- **File**: `frontend-deployment.yaml`
    - Define the Deployment for `hermesai-frontend`.

- **File**: `router-deployment.yaml`
    - Define the Deployment for `hermesai-router`.
    - Include health check configuration.

- **File**: `tunnel-deployment.yaml`
    - Define the Deployment for `hermesai-tunnel`.
    - Use `hostPath` for cloudflared configuration.

### 5. Services
- **File**: `services.yaml`
    - Define ClusterIP Services for `hermesai-db`, `hermesai-backend`, and `hermesai-frontend`.
    - Define LoadBalancer Service for `hermesai-router`.

### 6. Networking
- **File**: `networking.yaml` (if necessary)
    - Define Network Policies.
    - Integrate Service Mesh configurations.

## Additional Changes
- **Image Availability**: Ensure Docker images are pushed to a Container Registry.
- **Autoscaling**: Consider implementing `HorizontalPodAutoscaler` for each Deployment.
- **Ingress**: Deploy Ingress Controllers if needed for external traffic management.
- **Monitoring**: Integrate monitoring and logging tools (e.g., Prometheus, Grafana).

By implementing these changes, you'll be able to transition to a Kubernetes environment while retaining the key application configurations from your Docker Compose setup.

-------- LOCAL DEPLOY CONFIG ---------

The Kubernetes configuration outlined above is meant for deployment on a Kubernetes cluster. Here are some key points and suggestions on how you can manage and test Kubernetes configurations locally and how they compare to Docker Compose:

### Running Kubernetes Locally
- **Minikube**: You can use Minikube, which creates a local Kubernetes cluster on your machine for testing and development. This is similar to running Docker Compose locally with `docker-compose up`.
- **Kind (Kubernetes in Docker)**: Another tool is Kind, which runs Kubernetes clusters within Docker containers. This is lightweight and great for CI pipelines.

### Single Cluster Management Entry Points
- **kubectl**: Kubernetes doesn't have a direct equivalent to `docker-compose` for managing multiple resources as a single unit. However, you can execute commands using `kubectl` and apply all the configuration files at once by running:
  ```
  kubectl apply -f .
  ```
  You can put all your Kubernetes YAML files in a single directory and execute them together with the above command.

- **Helm**: Helm can be used to manage a collection of Kubernetes resources and deploy them as a single package. Helm charts are like advanced versions of Docker Compose files.

### Differences Between Kubernetes and Docker Compose
- **Complexity**: Kubernetes is designed for orchestration at a much larger scale than Docker Compose. It includes more complex resource definitions and management options.

- **Resource Types**: Kubernetes has different resource types like Deployments, Services, Ingresses, Secrets, ConfigMaps, etc., each handling different aspects of the infrastructure.

- **Scalability and Reliability**: Kubernetes is built with more scalability and reliability features in mind, including automated scaling, self-healing, and rolling updates.

- **CLI Tools**: The Kubernetes ecosystem uses `kubectl` and optionally `helm`, whereas Docker Compose uses the `docker-compose` CLI for interacting with applications.

### Recommendations for Local Development
- **Start with Minikube or Kind**: These tools will help you simulate a Kubernetes environment locally and test your configurations.
- **Use Helm for Packaging**: Consider adopting Helm charts for managing Kubernetes manifests as a single package, making deployment easier and more manageable.
- **Integrate CI/CD**: Use CI/CD pipelines to automate testing and deployment of your Kubernetes configurations.

These tools and practices will help you better understand and manage Kubernetes configurations locally and in production environments.




------ DOCKER COMPOSE OPTIONS -------


Here are your options for deploying a Docker Compose application to DigitalOcean:

### 1. Use DigitalOcean's App Platform
- **Direct Integration**: DigitalOcean's App Platform allows you to deploy directly from Docker images or source code repositories that include Dockerfiles, without converting to Kubernetes.
- **Managed Services**: You can take advantage of managed databases or storage provided by DigitalOcean, similar to a Platform as a Service (PaaS).

### 2. Convert to Kubernetes and Use DigitalOcean Kubernetes (DOKS)
- **Kubernetes Migration**: Convert your Docker Compose file to Kubernetes manifests and deploy using DigitalOcean's managed Kubernetes service.
- **Scalability and Management**: Gain the benefits of Kubernetes in terms of scalability, automated deployment, and management features.
- **Cost**: While DOKS offers a more comprehensive orchestration solution, ensure you're ready for the additional complexity and potential cost.

### 3. Deploy Docker Compose on Droplets
- **Manual Deployment**: You can manually provision DigitalOcean Droplets (VMs) and use SSH to deploy your application with Docker Compose.
- **Customizability**: Offers flexibility and simplicity if your application doesn't require orchestration beyond `docker-compose up`.
- **Scaling and Maintenance**: This approach requires manual scaling and maintenance.

### 4. Use DigitalOcean's Docker Machine Driver
- **Automated Provisioning**: Use Docker Machine with the DigitalOcean driver to provision and manage Droplets directly using Docker commands.
- **Integration**: It automates some aspects of deploying Docker containers on DigitalOcean.

### Summary of Options

- **Ease and Convenience**: Use the App Platform for ease of use and integrated services.
- **Scalability and Orchestration**: Use DOKS if you require sophisticated orchestration features.
- **Simplicity**: Use manual deployment on Droplets if you prefer simplicity and direct control without the complexities of Kubernetes.
- **Automated Droplet Provisioning**: Use Docker Machine if you prefer managing infrastructure through Docker CLI commands.

Your choice will depend on your application's requirements, your team's familiarity with the technologies, and the level of control or automation you desire.