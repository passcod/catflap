const app = require('express')()

app.get('/', (req, res) =>
  res.send('hello world')
)

app.listen({ fd: +process.env.LISTEN_FD })
