const WebSocket = require('websocket').client

async function main() {
  const url = 'ws://localhost:8888/'
  const players = ['ALEX', 'BOB', 'CHARLIE', 'DAVID', 'ED']
  const gameId = process.argv[2]
  
  if (!/[A-Z]{4}/.test(gameId)) {
    console.error('Not a valid game ID: ' + gameId)
    process.exit(1)
  }

  for (const name of players) {
    createPlayer(url, gameId, name)
    await wait(200)
  }
}

async function createPlayer(url, gameId, name) {
  console.log(`Creating player: ${name}`)

  const connect = () => {
    const ws = new WebSocket()
    ws.on('connect', conn => {
      let timeout
      conn.send(JSON.stringify({ type: 'player_join', gameId, name }))
      conn.on('message', msg => {
        const data = JSON.parse(msg.utf8Data)
        if (data.type == 'error') {
          console.error(data.error)
        }
        if (data.type !== 'update') return
        const action = data.state.action?.type
        clearTimeout(timeout)
        timeout = setTimeout(() => {
          const reply = react(data.state)
          if (reply == null) return
          conn.send(JSON.stringify({ type: 'player_action', action, data: reply }))
        }, 1000)
      })
      conn.on('close', reconnect)
    })
    ws.on('connectFailed', reconnect)
    ws.connect(url)
  }

  let reconnTimeout
  const reconnect = () => {
    clearTimeout(reconnTimeout)
    reconnTimeout = setTimeout(connect, 5000)
  }

  connect()
}

function react(msg) {
  const { action, players, role } = msg
  if (action == null) return
  if (action.type == 'lobby') {
    return action.canStart ? 'start' : undefined
  }
  if (action.type == 'nightRound') {
    return 'done'
  }
  if (action.type == 'choosePlayer') {
    const choice = action.players[Math.floor(Math.random() * action.players.length)]
    return players[choice].id
  }
  if (action.type == 'vote') {
    return Math.random() < 0.5 // 70% chance of "ja"
  }
  if (action.type == 'legislative') {
    if (action.canVeto && Math.random() < 0.5) {
      return { type: 'veto' }
    }
    const idx = Math.floor(Math.random() * action.cards.length)
    return { type: 'discard', idx }
  }
  if (action.type == 'nextRound') {
    return 'next'
  }
  console.error(`Unknown action type: ${action.type}`)
  // todo
}

function wait(ms) {
  return new Promise(res => setTimeout(res, ms))
}

main()