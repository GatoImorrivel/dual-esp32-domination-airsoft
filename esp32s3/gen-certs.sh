openssl req -x509 -nodes -days 36500 \
  -newkey rsa:2048 \
  -keyout ./tls/key.pem \
  -out ./tls/cert.pem \
  -config ./cert.conf

























