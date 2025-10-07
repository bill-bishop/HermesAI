DigitalOceanDeployment.md

# Deploying to Digital Ocean using Docker Compose

This guide provides steps on how to deploy your application using Docker Compose on a Digital Ocean droplet. The following instructions assume you already have a Digital Ocean account and are familiar with the basics of Docker and SSH.

## Prerequisites

1. **Digital Ocean Droplet**: Create a droplet using your preferred image. Ensure you select a droplet size that suits the resource requirements of your application.
2. **Domain Name**: (Optional) If you want to use a domain name, ensure it is set up with DNS records pointing to your droplet's IP.
3. **Docker and Docker Compose**: Ensure Docker and Docker Compose are installed on your droplet.

## Step-by-Step Setup

### 1. SSH into your Digital Ocean Droplet

Open a terminal and connect to your droplet via SSH:

```shell
ssh root@your_droplet_ip
```

### 2. Update and Install Prerequisites

Update packages and install any necessary dependencies:

```shell
apt update && apt upgrade -y
apt install -y docker docker-compose
```

### 3. Transfer your Application Files

Upload your docker-compose.yml and relevant application files to the droplet. You can use SCP, FTP, or a Git repository to transfer files.

```shell
scp -r /path_to_your_project root@your_droplet_ip:/path_on_server
```

### 4. Configure Environment Variables

Ensure that your environment variables, especially passwords and sensitive information, are securely managed. You can set them in an `.env` file or export them directly in the `docker-compose.yml` file.

```shell
# Example `.env` file
POSTGRES_PASSWORD=your_secure_password
DATABASE_URL=postgres://hermesai_user:your_secure_password@database:5432/hermesaidb
```

### 5. Start Docker Compose

Navigate to the directory containing your `docker-compose.yml` file and execute:

```shell
cd /path_to_your_project
docker-compose up -d
```

### 6. Verify Deployment

- Check the status of your services:

```shell
docker-compose ps
```

- Ensure there are no issues by viewing the logs:

```shell
docker-compose logs -f
```

- Confirm router healthcheck via:

```shell
curl http://localhost/api/healthcheck
```

### 7. Configure Domain (Optional)

If you are using a domain, ensure you have an A record pointing to your droplet’s IP.

### 8. Enable SSL (Optional)

To secure your connections with SSL, consider setting up Let's Encrypt for SSL certificates.

```shell
apt install certbot
certbot --nginx -d yourdomain.com
```

### Troubleshooting

- Ensure no port conflicts on the droplet are preventing Docker from binding.
- Check service logs for errors not directly visible with `docker-compose ps`.
- Verify connectivity to your database service and check that initial scripts execute correctly. Adjust your `init.sql` script if needed.

### Additional Considerations

- **Scaling**: Consider using a larger droplet or Digital Ocean’s Kubernetes services for scaling.
- **Monitoring**: Implement monitoring and alerting to detect any downtime or performance issues.
- **Backups**: Regularly back up your data; Digital Ocean provides snapshot options.

By completing these steps, your application should be running on a Digital Ocean droplet, fully set up with Docker Compose, and accessible via the specified domain or IP address.

You can run your full Docker Compose setup from a single droplet. Each service defined in your `docker-compose.yml` file will run as a separate container on that droplet. Ensure the droplet has sufficient resources (CPU, RAM, and storage) to handle all the containers in your setup. This is often a cost-effective choice for smaller applications or development environments. As your application grows or requires more resources, you might consider expanding to multiple droplets or using Digital Ocean’s Kubernetes service for more advanced scalability.