## Echo server used in other samples

- Deploy echo server:

```bash
kustomize build . | kubectl apply -f -
```

- Add entry to hosts file: `echo echo.localhost 127.0.0.1 | sudo tee -a /etc/hosts`

- Test echo server: `curl -vvv http://echo.localhost`
