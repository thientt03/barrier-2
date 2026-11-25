import { invoke } from '@tauri-apps/api/core'

// Backend trả lỗi dạng chuỗi tiếng Việt. Khi hết phiên đăng nhập, báo cho app đưa về màn hình login.
const UNAUTH_MARKER = 'đăng nhập lại'
let onUnauthenticated = () => {}
export function setUnauthenticatedHandler(fn) {
  onUnauthenticated = fn
}

async function call(cmd, args) {
  try {
    return await invoke(cmd, args)
  } catch (e) {
    const msg = typeof e === 'string' ? e : (e?.message ?? String(e))
    if (msg.includes(UNAUTH_MARKER)) onUnauthenticated()
    throw new Error(msg)
  }
}

export const api = {
  // Tài khoản
  needsSetup: () => call('needs_setup'),
  setupFirstManager: (input) => call('setup_first_manager', { input }),
  login: (username, password) => call('login', { username, password }),
  logout: () => call('logout'),
  currentUser: () => call('current_user'),

  // Thẻ (manager)
  listCards: () => call('list_cards'),
  createCard: (input) => call('create_card', { input }),

  // Đầu đọc thẻ cổng serial (manager)
  listSerialPorts: () => call('list_serial_ports'),
  getReaderConfig: () => call('get_reader_config'),
  setReaderConfig: (input) => call('set_reader_config', { input }),

  // Xe vào
  prepareEntry: (cardCode, lane) => call('prepare_entry', { cardCode, lane }),
  confirmEntry: (lane) => call('confirm_entry', { lane }),

  // Xe ra
  prepareExit: (cardCode, lane) => call('prepare_exit', { cardCode, lane }),
  confirmExit: (lane) => call('confirm_exit', { lane }),
  refuseExit: (lane) => call('refuse_exit', { lane }),

  // Dùng chung cho cả vào và ra
  recapture: (lane) => call('recapture', { lane }),
  cancelPending: (lane) => call('cancel_pending', { lane }),
}
