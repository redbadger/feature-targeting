# GraphQL backend for Todo MVC

Currently supports:

- get all todos
- get a single todo, by id
- add a todo
- update a todo
- delete a todo

Built with:

- [`async-graphql`](https://github.com/async-graphql/async-graphql) (for GraphQL)
- [`tide`](https://github.com/http-rs/tide) (for HTTP server)
- [`async-std`](https://docs.rs/async-std/1.6.2/async_std/) (uses [`smol`](https://github.com/stjepang/smol) for async runtime)
- [`sqlx`](https://github.com/launchbadge/sqlx) (for SQL queries)
- PostgreSQL (database)

---

_Note that a local instance of PostgreSQL is needed in order to compile._

- Because we use the `sqlx::query_as_file!()` macro (which validates queries at compile time against a schema in a PostgreSQL database), you should run something like the following to create a `todos` database on a local PostgreSQL instance:

  ```sh
  (
      set -euxo pipefail

      createdb -U ${USERNAME} todos || true

      DATABASE_URL=postgres://${USERNAME}@localhost/todos
      psql -d "${DATABASE_URL}" -f ./schema.sql
      psql -d "${DATABASE_URL}" -c 'CREATE EXTENSION IF NOT EXISTS "uuid-ossp";'

      echo "DATABASE_URL=${DATABASE_URL}" > .env
  )
  ```

- To build and run:

  ```sh
  cargo run
  ```

- Access the Graphiql UI at [http://localhost:3030/graphiql](http://localhost:3030/graphiql)

## CI Build

Currently we are tied to master branch of [sqlx](https://github.com/launchbadge/sqlx) in order to be able to use the cargo subcommand `cargo sqlx`. Clone the `sqlx` repo and install the subcommand:

```sh
git clone https://github.com/launchbadge/sqlx
cd sqlx
cargo install --path sqlx-cli
```

Ensure the SQL statements are `PREPARED` before building:

```sh
cargo sqlx prepare
```

Build the Docker image:

```sh
docker build . -t todomvc_api
```

Run the Docker image (Docker Desktop for Mac):

```sh
docker run --env DATABASE_URL=postgres://stuartharris@host.docker.internal/todos -it -p3030:3030 todomvc_api
```
