import React from 'react';

const SyncHistory = ({ history }) => {
  if (!history.length) {
    return <p className="empty">동기화 내역이 없습니다.</p>;
  }

  return (
    <ul className="history-list">
      {history.map((item) => (
        <li key={`${item.tool}-${item.synced_at}`} className={`history-item status-${item.status}`}>
          <div>
            <strong>{item.tool}</strong>
            <span className="status">{translateStatus(item.status)}</span>
          </div>
          <p>{item.message}</p>
          <time>{new Date(item.synced_at).toLocaleString()}</time>
        </li>
      ))}
    </ul>
  );
};

function translateStatus(status) {
  switch (status) {
    case 'Updated':
    case 'updated':
      return '갱신';
    case 'Skipped':
    case 'skipped':
      return '유지';
    default:
      return '실패';
  }
}

export default SyncHistory;
