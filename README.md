# Lojban Lens Search

This repository contains a web application for managing the Lojban dictionary/corpus and searching Lojban mailing list emails. It consists of a Rust backend API and a Vue.js frontend.

## Prerequisites

- Docker
- Docker Compose

## Getting Started

1. Clone this repository:
   ```
   git clone https://github.com/lojban/lensisku.git
   cd lensisku
   ```
2. Copy .env.example to .env:
   ```
   cp .env.example .env
   ```

### Running in Docker

Pay attention for MAILDIR_PATH variable in .env

#### Option 1: Using Docker Compose (Recommended)

Build and run the Docker container from the root of the cloned repository:

```
docker compose up --build
```

Or run in detached mode:

```
docker compose up -d --build
```

This will start the application along with all required services (database, infinity, etc.), and it will be accessible at `http://localhost:8051`.

**Required environment variables** (set in your `.env` file):
- `JWT_SECRET` - Secret key for JWT tokens
- `DB_USER` - PostgreSQL username
- `DB_PASSWORD` - PostgreSQL password  
- `DB_NAME` - PostgreSQL database name
- `DB_PORT` - PostgreSQL port (defaults to 5432)
- `MAILDIR_PATH` - Path to maildir directory

**Optional (local development):**
- `DISABLE_EMBEDDINGS=1` (or `true`/`yes`) - Skip loading the embedding model and all embedding computation; semantic search returns 503 with a message, and the background embedding job no-ops

#### Option 2: Using Makefile

The Makefile provides convenient commands for managing the Docker environment:

```bash
# Start the development environment
make up

# Rebuild Docker images
make build

# View logs from all containers
make logs

# List running containers
make ps

# Stop services
make down

# Clean up (remove containers and volumes)
make clean
```

#### Option 3: Direct Docker Commands

If you want to build and run the Dockerfile directly without docker-compose:

```bash
# Build the image
docker build -t lenisku:latest .

# Run the container
docker run -d \
  -p 8051:80 \
  -e DATABASE_URL=postgres://lojban:password@host.docker.internal/lojban_lens \
  -e JWT_SECRET=your-secret-key \
  -v ./maildir:/usr/src/app/maildir:ro \
  lenisku:latest
```

**Note:** When using direct Docker commands, you'll need to ensure:
- A PostgreSQL database is available and accessible
- The `DATABASE_URL` environment variable points to your database
- The maildir volume is properly mounted
- All required environment variables are set

### Alternative if docker-compose is unavailable

```
# Build the image
podman build -t mail_archive_app .

# Create and run the container
podman run -d \
  --name mail_archive_app \
  -p 8051:80 \
  -v ./mail_archive.db:/usr/src/app/mail_archive.db:Z \
  -e DATABASE_URL=sqlite:///usr/src/app/mail_archive.db \
  --restart unless-stopped \
  mail_archive_app
```

## Testing payments

### PayPal Payment Integration

### PayPal

Setup autoreturn

Go to your Account Settings.
Click Website payments.
Click Update across from "Website preferences."
Click On under "Auto Return."
In the Return URL field, enter the URL where you want to send your buyer after the payment.
The Return URL is applied to all Auto Return payments unless otherwise specified within the payment button or link.
Click Save.

Frontend developers should follow these steps to implement PayPal balance top-up:

1. Create a payment form with:
   - Amount input (minimum $0.50)
   - Currency selector (only USD supported)
   - PayPal payment button

2. When user submits form:
   - Call POST /payments with payload:
     ```json
     {
       "provider": "paypal",
       "amount_cents": 10000, // Amount in cents (e.g. $100.00 = 10000 cents)
       "currency": "USD"      // Must be "USD"
     }
     ```
   - Handle response:
     ```json
     {
       "payment_id": "PAYPAL-ORDER-ID",
       "redirect_url": "https://www.paypal.com/checkout/ORDER-ID"
     }
     ```

3. Redirect user to the returned redirect_url

4. Handle payment completion:
   - Option 1: Listen for websocket events on /ws/payments
   - Option 2: Poll GET /payments/balance periodically
   - Update UI with new balance when payment succeeds

5. Required environment variables:
   - PAYPAL_CLIENT_ID: Your PayPal client ID
   - PAYPAL_CLIENT_SECRET: Your PayPal client secret
   - PAYPAL_SANDBOX_MODE: Set to "true" for testing, "false" for production

Example frontend code:

```javascript
async function createPayPalPayment(amountCents) {
  const response = await fetch('/payments', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${userToken}`
    },
    body: JSON.stringify({
      provider: 'paypal',
      amount_cents: amountCents,
      currency: 'USD'
    })
  });
  
  const data = await response.json();
  if (response.ok) {
    window.location.href = data.redirect_url;
  } else {
    alert(`Payment failed: ${data.message}`);
  }
}
```

### Stripe

To test Stripe, you'll need to:

1. Set up Stripe CLI to forward webhooks to your local development server:

```
 stripe listen --forward-to localhost:8080/payments/webhook
```

2. Trigger test webhooks:

```
 stripe trigger payment_intent.succeeded
 stripe trigger payment_intent.payment_failed
```

The webhook handler will verify the signature using your webhook secret, then process the payment events accordingly. Successful payments will update both the payment status and user balance, while failed payments
will only update the payment status.

## Project Structure

- `src/`: Contains the Rust backend code
- `frontend/`: Contains the Vue.js frontend code
- `Dockerfile`: Defines the Docker image for the application
- `docker-compose.yml`: Defines the Docker Compose configuration

## Building For The Main Site

For building lensisku for hosting on the actual lojban.org
infrastructure, see building/README.txt

## Development

To develop the application locally:

0. Pre-install libs.
   For Ubuntu/Debian, run:

   ```
   sudo apt-get update && sudo apt-get install -y \
      build-essential \
      libharfbuzz-dev \
      libfreetype6-dev \
      libfontconfig1-dev \
      libicu-dev \
      zlib1g-dev \
      pkg-config \
      texlive-xetex \
      texlive-fonts-recommended \
      texlive-fonts-extra \
      texlive-latex-extra \
      texlive-lang-chinese \
      texlive-lang-japanese \
      fonts-noto-cjk fonts-noto-cjk-extra \
      fonts-linuxlibertine
   ```

   For Fedora/RHEL:

   ```
   sudo dnf install -y \
    @development-tools \
    harfbuzz-devel \
    freetype-devel \
    fontconfig-devel \
    libicu-devel \
    zlib-devel \
    pkg-config \
    texlive-xetex \
    texlive-collection-fontsrecommended \
    texlive-collection-fontsextra \
    texlive-collection-latexextra \
    texlive-collection-langchinese \
    texlive-collection-langjapanese \
    google-noto-cjk-fonts \
    linux-libertine-fonts
   ```

1. For the backend:

   ```
   docker compose -f ./docker-compose.dev.yml up -d
   make back
   ```

2. For the frontend:

   ```
   cd frontend && pnpm i && cd ..
   make front
   ```

   Then access the frontend at http://localhost:5173/

### API Documentation

API documentation is available at `http://localhost:8080/swagger-ui/` when the application is running.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License.
