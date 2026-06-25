import { useState } from 'react'
import { invoke } from '../../services/tauri'
import { useBackupStore } from '../../stores/backupStore'
import type { FriendContact } from '../../types'
import { useI18n } from '../../i18n/useI18n'

export default function FriendContacts() {
  const {
    friendContacts,
    activePingContactId,
    addFriendContact,
    updateFriendContact,
    removeFriendContact,
    setActivePingContactId,
  } = useBackupStore()
  const { tk } = useI18n()

  const [editingField, setEditingField] = useState<{ id: string; field: 'name' | 'ip' } | null>(null)
  const [editValue, setEditValue] = useState('')

  const handlePing = async (contact: FriendContact) => {
    if (activePingContactId === contact.id) {
      try { await invoke('stop_ping') } catch { /* ignore */ }
      setActivePingContactId(null)
      localStorage.removeItem('ark-ping-active')
      localStorage.removeItem('ark-ping-ip')
    } else {
      if (activePingContactId !== null) {
        try { await invoke('stop_ping') } catch { /* ignore */ }
      }
      try {
        await invoke('start_ping', { ip: contact.ip })
        setActivePingContactId(contact.id)
        localStorage.setItem('ark-ping-ip', contact.ip)
        localStorage.setItem('ark-ping-active', 'true')
      } catch { /* ignore — ip might be empty */ }
    }
  }

  const startEdit = (id: string, field: 'name' | 'ip', current: string) => {
    setEditingField({ id, field })
    setEditValue(current)
  }

  const commitEdit = () => {
    if (!editingField) return
    updateFriendContact(editingField.id, { [editingField.field]: editValue.trim() })
    setEditingField(null)
  }

  const handleAddContact = () => {
    addFriendContact({ name: tk('default_contact_name', 'Friend'), ip: '' })
  }

  const handleRemove = async (id: string) => {
    if (activePingContactId === id) {
      try { await invoke('stop_ping') } catch { /* ignore */ }
      setActivePingContactId(null)
      localStorage.removeItem('ark-ping-active')
      localStorage.removeItem('ark-ping-ip')
    }
    removeFriendContact(id)
  }

  return (
    <div className="ark-panel rounded-lg p-4 space-y-3">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <span className="text-ark-cyan/70 text-xs font-bold tracking-widest uppercase">
            {tk('friend_contacts', 'Friend Book')}
          </span>
          {activePingContactId && (
            <span className="flex items-center gap-1">
              <span className="w-1.5 h-1.5 rounded-full bg-green-400 animate-pulse inline-block" />
              <span className="text-green-400/70 text-[10px] font-bold tracking-widest">{tk('ping_active', 'PING ACTIVE')}</span>
            </span>
          )}
        </div>
        <button
          onClick={handleAddContact}
          className="ark-action-btn text-[10px] px-2.5 py-0.5 flex items-center gap-1"
        >
          {tk('add_friend', '+ Add')}
        </button>
      </div>

      {/* Contact list */}
      {friendContacts.length === 0 ? (
        <p className="text-ark-cyan/25 text-xs py-2 text-center">
          {tk('no_contacts', 'No contacts — add your friends\' IPs with the + button')}
        </p>
      ) : (
        <div className="space-y-1.5">
          {friendContacts.map((contact) => {
            const isPinging = activePingContactId === contact.id
            return (
              <div
                key={contact.id}
                className="flex items-center gap-2 px-2.5 py-1.5 rounded"
                style={{
                  background: isPinging ? 'rgba(74,222,128,0.06)' : 'rgba(255,255,255,0.03)',
                  border: `1px solid ${isPinging ? 'rgba(74,222,128,0.3)' : 'rgba(255,255,255,0.07)'}`,
                }}
              >
                {/* Name */}
                {editingField?.id === contact.id && editingField.field === 'name' ? (
                  <input
                    autoFocus
                    className="flex-shrink-0 w-24 bg-transparent border border-ark-cyan/40 text-ark-cyan/90 text-xs px-1.5 py-0.5 rounded focus:outline-none font-mono"
                    value={editValue}
                    onChange={(e) => setEditValue(e.target.value)}
                    onBlur={commitEdit}
                    onKeyDown={(e) => { if (e.key === 'Enter') commitEdit() }}
                  />
                ) : (
                  <span
                    className="flex-shrink-0 w-24 text-xs font-semibold cursor-pointer truncate"
                    style={{ color: isPinging ? 'rgba(74,222,128,0.9)' : 'rgba(255,255,255,0.7)' }}
                    onClick={() => startEdit(contact.id, 'name', contact.name)}
                  >
                    {contact.name || tk('no_name', 'no name')}
                  </span>
                )}

                {/* IP */}
                {editingField?.id === contact.id && editingField.field === 'ip' ? (
                  <input
                    autoFocus
                    className="flex-1 bg-transparent border border-ark-cyan/40 text-ark-cyan/90 text-xs px-1.5 py-0.5 rounded focus:outline-none font-mono"
                    value={editValue}
                    onChange={(e) => setEditValue(e.target.value)}
                    onBlur={commitEdit}
                    onKeyDown={(e) => { if (e.key === 'Enter') commitEdit() }}
                    placeholder="100.x.x.x"
                  />
                ) : (
                  <span
                    className="flex-1 text-xs font-mono cursor-pointer truncate"
                    style={{ color: isPinging ? 'rgba(74,222,128,0.7)' : 'rgba(0,200,255,0.5)' }}
                    onClick={() => startEdit(contact.id, 'ip', contact.ip)}
                  >
                    {contact.ip || <span style={{ color: 'rgba(255,255,255,0.2)' }}>{tk('no_ip_short', 'no IP')}</span>}
                  </span>
                )}

                {/* Ping button */}
                <button
                  onClick={() => handlePing(contact)}
                  disabled={!contact.ip}
                  className="flex-shrink-0 text-[9px] font-bold tracking-widest px-2 py-0.5 rounded transition-all"
                  style={isPinging ? {
                    background: 'rgba(239,68,68,0.15)',
                    color: 'rgba(239,68,68,0.8)',
                    border: '1px solid rgba(239,68,68,0.35)',
                  } : {
                    background: 'rgba(0,200,255,0.08)',
                    color: contact.ip ? 'rgba(0,200,255,0.7)' : 'rgba(255,255,255,0.15)',
                    border: '1px solid rgba(0,200,255,0.2)',
                    opacity: contact.ip ? 1 : 0.4,
                  }}
                >
                  {isPinging ? '■ STOP' : '▶ PING'}
                </button>

                {/* Delete */}
                <button
                  onClick={() => handleRemove(contact.id)}
                  className="flex-shrink-0 text-[10px] px-1.5 py-0.5 rounded transition-colors"
                  style={{ color: 'rgba(239,68,68,0.4)' }}
                >
                  ×
                </button>
              </div>
            )
          })}
        </div>
      )}

      <p className="text-ark-cyan/25 text-[10px]">
        {tk('footer_contacts_hint', 'Click name or IP to edit · Ping keeps the Tailscale route active')}
      </p>
    </div>
  )
}
