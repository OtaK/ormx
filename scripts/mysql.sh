CONTAINER_ID=$(
  docker run -it --rm --name ormx-test-mysql-db \
    -e MYSQL_DATABASE=ormx \
    -e MYSQL_ROOT_PASSWORD=admin \
    -v $(pwd)/scripts/mysql-schema.sql:/docker-entrypoint-initdb.d/schema.sql \
    -d mysql
)
CONTAINER_IP=$(docker inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' $CONTAINER_ID)
echo "DATABASE_URL=mysql://root:admin@$CONTAINER_IP/ormx" > .env
docker attach $CONTAINER_ID
