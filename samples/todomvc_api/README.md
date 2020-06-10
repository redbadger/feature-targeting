```sh
(
    set -euxo pipefail
    createdb -U ${USERNAME} todos || true
    export DATABASE_URL=postgres://${USERNAME}@localhost/todos
    psql -d "${DATABASE_URL}" -f ./schema.sql
    echo "DATABASE_URL=${DATABASE_URL}" > .env
)
```
