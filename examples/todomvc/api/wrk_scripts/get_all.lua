wrk.method = "POST"
wrk.body   = ' { todos { id title completed } } '
wrk.headers["Content-Type"] = "application/json"
wrk.headers["Authorization"] = "Bearer jwt-token-goes-here"
