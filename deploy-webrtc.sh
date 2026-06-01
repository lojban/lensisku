#!/bin/bash

# Deployment script for WebRTC/WebSocket support in Lensisku

set -e

echo "🚀 Deploying WebRTC/WebSocket support for Lensisku..."

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
    echo "📜 Generating self-signed SSL certificates..."
    sudo mkdir -p /etc/ssl/certs /etc/ssl/private
    sudo openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
        -keyout /etc/ssl/private/localhost.key \
        -out /etc/ssl/certs/localhost.crt \
        -subj "/C=US/ST=CA/L=SanFrancisco/O=Lensisku/OU=WebRTC/CN=localhost"
fi

# Generate DH parameters
if [ ! -f "/etc/ssl/certs/dhparam.pem" ]; then
    echo "🔐 Generating DH parameters..."
    sudo openssl dhparam -out /etc/ssl/certs/dhparam.pem 2048
fi

# Start coturn server
echo "🌐 Starting coturn STUN/TURN server..."
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

# Update nginx configuration
echo "🔧 Updating nginx configuration..."
# Copy the updated nginx config to the container location
sudo cp /home/user/lojban/lensisku/archive/lensisku-containers/misc/nginx.conf /etc/nginx/nginx.conf

# Test nginx configuration
sudo nginx -t

# Reload nginx
sudo systemctl reload nginx

echo "✅ Nginx configuration updated and reloaded"

# Add WebRTC environment variables to .env
echo "📝 Adding WebRTC environment variables..."
cat >> .env << EOF

# WebRTC Configuration
TURN_SERVER_URL=turn:lensisku.lojban.org:3478
TURN_USERNAME=lensisku
TURN_SECRET=$TURN_SECRET
STUN_SERVER_URL=stun:lensisku.lojban.org:3478
WEBRTC_ENABLED=true
EOF

echo "🎉 WebRTC/WebSocket deployment completed!"
echo ""
echo "📋 Next steps:"
echo "1. Restart the lensisku application"
echo "2. Test WebSocket connections at ws://localhost/ws"
echo "3. Test WebRTC signaling at /webrtc/signal"
echo "4. Update frontend to use TURN server: $TURN_SERVER_URL"
echo ""
echo "🔧 STUN/TURN Server Status:"
docker-compose -f docker-compose.webrtc.yml ps
