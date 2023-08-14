#!/bin/sh

# Wait for the fullchain.pem file to be created
while [ ! -f /etc/letsencrypt/live/farani.dev/fullchain.pem ]; do
  echo "Waiting for fullchain.pem..."
  sleep 1
done

echo "fullchain.pem found, starting nginx..."
# Start nginx
exec nginx -g "daemon off;"