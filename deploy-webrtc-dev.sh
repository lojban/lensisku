#!/bin/bash

# Development deployment script for WebRTC/WebSocket support in Lensisku

set -e

echo "🚀 Deploying WebRTC/WebRTC support for Lensisku Development Environment..."

# Check if docker-compose is available
if ! command -v docker-compose &> /dev/null; then
    echo "❌ docker-compose is not installed"
    exit 1
fi

# Generate TURN secret if not set
if [ -z "$TURN_SECRET" ]; then
    export TURN_SECRET=$(openssl rand -hex 32)
    echo "🔑 Generated TURN secret: $TURN_SECRET"
fi

# Create SSL certificates for coturn (self-signed for development)
if [ ! -f "/etc/ssl/certs/localhost.crt" ]; then
    echo "📜 Generating self-signed SSL certificates for development..."
    sudo mkdir -p /etc/ssl/certs /etc/ssl/private
    sudo openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
        -keyout /etc/ssl/private/localhost.key \
        -out /etc/ssl/certs/localhost.crt \
        -subj "/C=US/ST=CA/L=SanFrancisco/O=Lensisku-Dev/OU=WebRTC/CN=localhost"
fi

# Generate DH parameters
if [ ! -f "/etc/ssl/certs/dhparam.pem" ]; then
    echo "🔐 Generating DH parameters..."
    sudo openssl dhparam -out /etc/ssl/certs/dhparam.pem 2048
fi

# Start coturn server for development
echo "🌐 Starting coturn STUN/TURN server for development..."
docker-compose -f docker-compose.webrtc.yml up -d

# Wait for coturn to start
echo "⏳ Waiting for coturn to start..."
sleep 5

# Test coturn server
echo "🧪 Testing coturn server..."
if nc -z localhost 3478; then
    echo "✅ Coturn server is running on port 3478"
else
    echo "❌ Coturn server failed to start"
    docker-compose -f docker-compose.webrtc.yml logs coturn
    exit 1
fi

# Update development nginx configuration
echo "🔧 Updating development nginx configuration..."
# Copy the updated nginx dev config to the container location
sudo cp /home/user/lojban/lensisku/archive/lensisku-containers/misc/nginx-dev.conf /etc/nginx/nginx-dev.conf

# Test nginx configuration
sudo nginx -t -c /etc/nginx/nginx-dev.conf

# Note: User needs to manually reload nginx with dev config
echo "⚠️  To apply development nginx config, run:"
echo "   sudo nginx -s reload -c /etc/nginx/nginx-dev.conf"
echo "   Or restart nginx with: sudo systemctl restart nginx"

echo "✅ Development nginx configuration updated"

# Add WebRTC environment variables to .env.dev
echo "📝 Adding WebRTC environment variables to .env.dev..."
cat >> .env.dev << EOF

# WebRTC Configuration (Development)
TURN_SERVER_URL=turn:localhost:3478
TURN_USERNAME=lensisku
TURN_SECRET=$TURN_SECRET
STUN_SERVER_URL=stun:localhost:3478
WEBRTC_ENABLED=true
EOF

echo "🎉 WebRTC/WebRTC development deployment completed!"
echo ""
echo "📋 Development Setup:"
echo "1. Development server runs on port: 20390"
echo "2. WebSocket endpoints: ws://localhost:20390/ws/"
echo "3. WebRTC signaling: http://localhost:20390/webrtc/"
echo "4. Health check: http://localhost:20390/messaging/health"
echo ""
echo "🔧 STUN/TURN Server Status:"
docker-compose -f docker-compose.webrtc.yml ps
echo ""
echo "🌐 Development URLs:"
echo "- Frontend: http://localhost:20390"
echo "- API: http://localhost:20390/api/"
echo "- Health: http://localhost:20390/messaging/health"
