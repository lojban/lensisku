# WebRTC and WebSocket Support for Lensisku

This document describes the WebRTC and WebSocket implementation for real-time messaging in Lensisku.

## Overview

The deployment now includes:
- **WebSocket Support**: Real-time bidirectional communication for messaging
- **WebRTC Support**: Peer-to-peer audio/video calling with STUN/TURN servers
- **Enhanced Security**: Proper CORS headers and authentication
- **Health Monitoring**: Health check endpoints for service status

## Architecture

```
Frontend (Vue.js)
    ↓ WebSocket/HTTP
Nginx (Reverse Proxy)
    ↓ HTTP/WebSocket
Lensisku Backend (Rust/Actix)
    ↓ Signaling
Coturn (STUN/TURN Server)
    ↓ P2P
Peers (WebRTC)
```

## Deployment

### Prerequisites
- Docker and Docker Compose
- OpenSSL for certificate generation
- Nginx configuration access

### Quick Start

1. **Deploy WebRTC Infrastructure:**
   ```bash
   ./deploy-webrtc.sh
   ```

2. **Restart Application:**
   ```bash
   # Restart your lensisku application to pick up new routes
   systemctl restart lensisku
   ```

3. **Verify Deployment:**
   ```bash
   curl http://localhost:20380/messaging/health
   ```

## Configuration

### Environment Variables

Add these to your `.env` file:

```bash
# WebRTC Configuration
TURN_SERVER_URL=turn:lensisku.lojban.org:3478
TURN_USERNAME=lensisku
TURN_SECRET=<generated-secret>
STUN_SERVER_URL=stun:lensisku.lojban.org:3478
WEBRTC_ENABLED=true
```

### Nginx Configuration

The nginx configuration has been updated to support:
- WebSocket upgrade handling
- WebRTC-specific security headers
- Proper proxy configuration for real-time protocols

### STUN/TURN Server

Coturn server provides:
- **STUN**: NAT traversal for peer discovery
- **TURN**: Relay server for restrictive NAT environments
- **TLS**: Secure connections on port 5349

## API Endpoints

### WebSocket Endpoints

- `GET /messaging/ws` - General WebSocket connection
- `GET /messaging/ws/{thread_id}` - Thread-specific WebSocket

### WebRTC Signaling Endpoints

- `POST /messaging/webrtc/signal` - Send WebRTC signal
- `GET /messaging/webrtc/signals/{user_id}` - Get pending signals
- `PUT /messaging/webrtc/signal/{signal_id}/processed` - Mark signal processed

### Health Check

- `GET /messaging/health` - Service health status

## Frontend Integration

### WebSocket Connection

```javascript
const ws = new WebSocket('ws://localhost:20380/messaging/ws/123');

ws.onopen = function(event) {
    console.log('WebSocket connected');
};

ws.onmessage = function(event) {
    const message = JSON.parse(event.data);
    console.log('Received:', message);
};
```

### WebRTC Configuration

```javascript
const pc = new RTCPeerConnection({
    iceServers: [
        { urls: 'stun:lensisku.lojban.org:3478' },
        { 
            urls: 'turn:lensisku.lojban.org:3478',
            username: 'lensisku',
            credential: '<turn-secret>'
        }
    ]
});
```

## Security Considerations

### Authentication
- All WebSocket connections require valid JWT tokens
- WebRTC signaling endpoints are protected
- User isolation enforced

### CORS Headers
- Proper CORS configuration for WebRTC
- WebSocket upgrade headers allowed
- Cross-origin security policies

### Network Security
- TURN server uses authentication
- TLS support for secure connections
- Rate limiting on signaling endpoints

## Monitoring

### Health Check Response

```json
{
  "status": "healthy",
  "timestamp": "2024-06-01T20:00:00Z",
  "services": {
    "websocket": {
      "status": "enabled",
      "endpoints": [
        "/messaging/ws",
        "/messaging/ws/{thread_id}"
      ]
    },
    "webrtc": {
      "status": "configured",
      "endpoints": [
        "/messaging/webrtc/signal",
        "/messaging/webrtc/signals/{user_id}",
        "/messaging/webrtc/signal/{signal_id}/processed"
      ],
      "stun_server": "stun:lensisku.lojban.org:3478",
      "turn_server": "turn:lensisku.lojban.org:3478"
    }
  }
}
```

### Logs

- Coturn logs: `/var/log/turnserver.log`
- Application logs: Check your application log location
- Nginx logs: `/var/log/nginx/access.log` and `/var/log/nginx/error.log`

## Troubleshooting

### Common Issues

1. **WebSocket Connection Failed**
   - Check nginx configuration
   - Verify JWT token validity
   - Check CORS headers

2. **WebRTC Connection Failed**
   - Verify coturn server is running
   - Check firewall rules for ports 3478, 5349, 49152-65535
   - Verify TURN credentials

3. **Audio/Video Not Working**
   - Check browser permissions
   - Verify ICE candidates
   - Check network connectivity

### Debug Commands

```bash
# Check coturn status
docker-compose -f docker-compose.webrtc.yml ps

# View coturn logs
docker-compose -f docker-compose.webrtc.yml logs coturn

# Test STUN server
turnutils_uclient -T -u lensisku -w <secret> localhost

# Test WebSocket
curl -i -N -H "Connection: Upgrade" \
     -H "Upgrade: websocket" \
     -H "Sec-WebSocket-Key: test" \
     -H "Sec-WebSocket-Version: 13" \
     http://localhost:20380/messaging/ws
```

## Performance Considerations

### Scaling WebSocket Connections
- Use Redis pub/sub for multi-instance deployments
- Implement connection pooling
- Monitor memory usage per connection

### WebRTC Optimization
- Use appropriate codec settings
- Implement bandwidth limits
- Monitor TURN server usage

### Resource Limits
- Coturn configured for 100 concurrent users
- WebSocket timeout set to 24 hours
- Rate limiting on signaling endpoints

## Future Enhancements

1. **Full WebSocket Implementation**: Replace placeholder handlers with full actor-based implementation
2. **SIP Integration**: Add SIP gateway for external calling
3. **Recording**: Implement call recording functionality
4. **Analytics**: Add usage metrics and analytics
5. **Load Balancing**: Configure multiple TURN servers for high availability

## Support

For issues with WebRTC/WebSocket deployment:
1. Check the health endpoint: `/messaging/health`
2. Review application and coturn logs
3. Verify network connectivity and firewall rules
4. Test with the provided debug commands
