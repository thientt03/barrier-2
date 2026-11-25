const money = new Intl.NumberFormat('vi-VN')

export const formatMoney = (n) => `${money.format(n)}đ`

export function formatDuration(minutes) {
  const h = Math.floor(minutes / 60)
  const m = minutes % 60
  if (h === 0) return `${m} phút`
  return m === 0 ? `${h} giờ` : `${h} giờ ${m} phút`
}

/** "2026-11-25 08:15:32" -> "08:15:32" */
export const timeOnly = (s) => (s ? s.slice(11) : '')

export const CARD_TYPE_LABEL = { monthly: 'VÉ THÁNG', guest: 'KHÁCH VÃNG LAI' }

/** VIP = vé tháng có ghi chú chứa "VIP" (spec 2.4). */
export const isVip = (card) => /vip/i.test(card.notes || '')
export const VEHICLE_LABEL = { motorbike: 'Xe máy', car: 'Ô tô', truck: 'Xe tải' }
