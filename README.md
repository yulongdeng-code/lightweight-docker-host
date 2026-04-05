# Lightweight Secure Docker Container Hosting Service
One-click deployment solution for Vultr Marketplace, designed for developers and small businesses.

## 🚀 Core Features
- One-click Docker Engine & Docker Compose deployment with zero manual configuration
- Built-in security hardening (Fail2ban brute-force protection, Nginx reverse proxy)
- Debian 12 minimal base image, lightweight and fast, low resource consumption
- Full compatibility with Vultr global cloud infrastructure and all instance specifications
- Automated service startup and health check, 24/7 stable operation
- Long-term maintenance and regular security updates, continuous feature iteration

## 📋 Technical Specifications
| Component | Version | Description |
|-----------|---------|-------------|
| Base OS   | Debian 12 Slim | Minimal, secure, and stable base system |
| Docker Engine | Latest stable | Industry-standard container runtime |
| Docker Compose | Plugin v2 | Multi-container orchestration support |
| Nginx | Latest stable | Reverse proxy and web service support |
| Fail2ban | Latest stable | Brute-force attack protection |

## 📦 Vultr Marketplace Deployment Guide
### 1. One-Click Deployment
1.  Log in to your Vultr account, navigate to the Marketplace
2.  Search for "Lightweight Secure Docker Container Hosting Service"
3.  Select your preferred Vultr data center and instance specification (minimum 2 vCPU / 4GB RAM recommended)
4.  Click "Deploy Now" to complete the one-click deployment

### 2. Post-Deployment Initialization
- The system will automatically initialize all services (Docker, Nginx, Fail2ban) on first boot
- No manual configuration required, the entire process takes 2-3 minutes
- You can access the server via SSH to verify service status

### 3. Service Verification
```bash
# Check Docker service status
docker --version
docker compose version

# Check Nginx status
systemctl status nginx

# Check Fail2ban status
systemctl status fail2ban
