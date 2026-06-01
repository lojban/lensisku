# WebRTC and WebSocket Development Environment for Lensisku

This document describes the development environment setup for WebRTC and WebSocket real-time messaging in Lensisku.

## Development vs Production

The development environment uses different ports and configurations to avoid conflicts with production:

| Service | Production | Development |
|---------|------------|-------------|
| Frontend | :20380 | :20390 |
| STUN Server | :3478 | :3479 |
| TURN TLS | :5349 | :5350 |
| TURN Realm | lensisku.lojban.org | localhost |

## Quick Start

### 1. Deploy Development WebRTC Infrastructure

```bash
./deploy-webrtc-dev.sh
```

### 2. Start Development Server

```bash
# Start lensisku in development mode
cargo run

# Or if using a specific development script
./run-dev-server.sh
```

### 3. Configure Nginx for Development

```bash
# Apply development nginx configuration
sudo nginx -s reload -c /etc/nginx/nginx-dev.conf

# Or restart nginx completely
sudo systemctl restart nginx
```

### 4. Verify Development Setup

```bash
# Check health endpoint
curl http://localhost:20390/messaging/health

# Test STUN server
turnutils_uclient -T -u lensisku-dev -w dev-secret localhost 3479
```

## Development Configuration

### Environment Variables (.env.dev)

```bash
# WebRTC Configuration (Development)
TURN_SERVER_URL=turn:localhost:3479
TURN_USERNAME=lensisku-dev
TURN_SECRET=lensisku-dev-turn-secret-key
STUN_SERVER_URL=stun:localhost:3479
WEBRTC_ENABLED=true

# Development Database
DATABASE_URL=postgresql://lensisku:password@localhost:5432/lensisku_dev

# Development Redis
REDIS_URL=redis://localhost:6379/1
```

### Frontend Development Configuration

```javascript
// config/dev.js
export const config = {
  apiUrl: 'http://localhost:20390/api',
  wsUrl: 'ws://localhost:20390/ws',
  webrtc: {
    iceServers: [
      { urls: 'stun:localhost:3479' },
      { 
        urls: 'turn:localhost:3479',
        username: 'lensisku-dev',
        credential: 'lensisku-dev-turn-secret-key'
      }
    ]
  }
};
```

## Development Endpoints

### WebSocket Endpoints
- `ws://localhost:20390/ws` - General WebSocket connection
- `ws://localhost:20390/ws/{thread_id}` - Thread-specific WebSocket

### WebRTC Signaling Endpoints
- `POST http://localhost:20390/webrtc/signal` - Send WebRTC signal
- `GET http://localhost:20390/webrtc/signals/{user_id}` - Get pending signals
- `PUT http://localhost:20390/webrtc/signal/{signal_id}/processed` - Mark signal processed

### Health Check
- `GET http://localhost:20390/messaging/health` - Service health status

## Development Features

### 1. Anonymous Access (Development Only)
The development coturn server allows anonymous access for easier testing:

```bash
# Test with any WebRTC client without authentication
# Uses anonymous credentials for development
```

### 2. Lower Resource Limits
- Concurrent users: 50 (vs 100 in production)
- Per-user quota: 6 (vs 12 in production)
- Bandwidth: 32kbps (vs 64kbps in production)

### 3. Debug Logging
- Verbose coturn logging enabled
- Development-specific log files
- Real-time signal debugging

## Testing WebRTC in Development

### 1. Simple Peer Connection Test

```javascript
// Create peer connection with development STUN/TURN
const pc = new RTCPeerConnection({
    iceServers: [
        { urls: 'stun:localhost:3479' },
        { 
            urls: 'turn:localhost:3479',
            username: 'lensisku-dev',
            credential: 'lensisku-dev-turn-secret-key'
        }
    ]
});

// Test connection
pc.createOffer().then(offer => {
    console.log('Development offer created:', offer);
});
```

### 2. WebSocket Connection Test

```javascript
// Connect to development WebSocket
const ws = new WebSocket('ws://localhost:20390/ws/123');

ws.onopen = function(event) {
    console.log('Development WebSocket connected');
    ws.send(JSON.stringify({
        type: 'test',
        message: 'Development test message'
    }));
};
```

### 3. Health Check Test

```bash
# Check all services are healthy
curl http://localhost:20390/messaging/health | jq

# Expected response:
{
  "status": "healthy",
  "timestamp": "2024-06-01T20:00:00Z",
  "services": {
    "websocket": {
      "status": "enabled",
      "endpoints": ["/messaging/ws", "/messaging/ws/{thread_id}"]
    },
    "webrtc": {
      "status": "configured",
      "stun_server": "stun:localhost:3479",
      "turn_server": "turn:localhost:3479"
    }
  }
}
```

## Development Debugging

### 1. Coturn Logs

```bash
# View development coturn logs
docker-compose -f docker-compose.webrtc-dev.yml logs coturn-dev

# Follow logs in real-time
docker-compose -f docker-compose.webrtc-dev.yml logs -f coturn-dev
```

### 2. Nginx Logs

```bash
# Development nginx access logs
tail -f /var/log/nginx/access.log

# Development nginx error logs
tail -f /var/log/nginx/error.log
```

### 3. Application Logs

```bash
# Development application logs
tail -f /var/log/lensisku/dev.log

# Or if running directly
cargo run 2>&1 | tee dev.log
```

## Common Development Issues

### 1. Port Conflicts
If you get port conflicts, check what's using the ports:

```bash
# Check STUN/TURN ports
netstat -tulnp | grep 3479
netstat -tulnp | grep 5350

# Check development server port
netstat -tulnp | grep 20390
```

### 2. Certificate Issues
Development uses self-signed certificates. Ignore browser warnings:

```bash
# Regenerate development certificates
sudo rm /etc/ssl/certs/localhost.crt
./deploy-webrtc-dev.sh
```

### 3. WebSocket Connection Issues
Check nginx configuration:

```bash
# Test nginx configuration
sudo nginx -t -c /etc/nginx/nginx-dev.conf

# Check WebSocket upgrade headers
curl -i -N -H "Connection: Upgrade" \
     -H "Upgrade: websocket" \
     http://localhost:20390/ws
```

## Development vs Production Differences

| Feature | Development | Production |
|---------|-------------|------------|
| Authentication | Anonymous + JWT | JWT only |
| TLS | Self-signed | Proper certificates |
| Logging | Debug level | Info level |
| Resource Limits | Lower | Higher |
| Domain | localhost | lensisku.lojban.org |
| Ports | Different | Standard |

## Switching Between Environments

### To Production
```bash
# Stop development services
docker-compose -f docker-compose.webrtc-dev.yml down

# Start production services
./deploy-webrtc.sh

# Use production nginx config
sudo nginx -s reload -c /etc/nginx/nginx.conf
```

### To Development
```bash
# Stop production services
docker-compose -f docker-compose.webrtc.yml down

# Start development services
./deploy-webrtc-dev.sh

# Use development nginx config
sudo nginx -s reload -c /etc/nginx/nginx-dev.conf
```

## Performance Testing in Development

### Load Testing WebRTC
```bash
# Test multiple concurrent connections
for i in {1..10}; do
    turnutils_uclient -T -u lensisku-dev -w dev-secret localhost 3479 &
done
```

### WebSocket Stress Test
```javascript
// Create multiple WebSocket connections
for (let i = 0; i < 10; i++) {
    const ws = new WebSocket(`ws://localhost:20390/ws/${i}`);
    ws.onopen = () => console.log(`Connection ${i} established`);
}
```

## Next Steps

1. **Implement Full WebSocket**: Replace placeholder handlers with full actor-based implementation
2. **Add WebRTC Recording**: Implement call recording for development testing
3. **Create Test Suite**: Add automated tests for WebRTC/WebSocket functionality
4. **Performance Monitoring**: Add metrics collection for development optimization

## Support

For development environment issues:
1. Check development health endpoint: `http://localhost:20390/messaging/health`
2. Review development logs in all services
3. Verify port usage with `netstat`
4. Test with the provided debug commands
