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

- Because we use the `sqlx::query_as_file!()` macro (which validates queries at compile time against a schema in a PostgreSQL database), you should run `make prepare` to run PostgresQL in Docker and migrate the schema.

- To build and run:

  ```sh
  make
  ```

- Access the Graphiql UI at [http://localhost:3030](http://localhost:3030). You will need a JWT token with an `email` claim in the `Authorization` header, e.g. add something like this to the "HTTP headers" section in the bottom left of GraphQL Playground:

  ```json
  {
    "Authorization": "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyLCJlbWFpbCI6InRlc3RAZXhhbXBsZS5jb20ifQ.Y4icSuhBb2U3U_tifd3YevJBpRtmh6OSHOLqX0RqINk"
  }
  ```

## CI Build

Currently we are tied to master branch of [sqlx](https://github.com/launchbadge/sqlx) in order to be able to use the cargo subcommand `cargo sqlx`.

- Ensure the SQL statements are validated before building:

  ```sh
  make prepare
  ```

  (note that if there are changes to `sqlx-data.json`, then you will need to commit and push them to the repository).

- Build the Docker image:

  ```sh
  make docker
  ```

- Run the Docker image (Docker Desktop for Mac):

  ```sh
  docker run --env DATABASE_URL=postgres://postgres@host.docker.internal/todos -it -p3030:3030 todomvc_api
  ```

## Running in Kubernetes

There are a set of [manifests](./manifests) in the `manifests` directory. To install on Docker for Mac:

```sh
(cd manifests && make)
```

You should be able to access the API at http://todo.red-badger.com/graphql (but you may need to add `todo.red-badger.com` to your hosts file in order to resolve).
