import React from 'react';

const MasterConfigEditor = ({ draft, onChange, onSave, loading }) => {
  return (
    <div className="master-config">
      <textarea
        value={draft}
        onChange={(event) => onChange(event.target.value)}
        spellCheck={false}
        rows={20}
      />
      <button className="primary" onClick={onSave} disabled={loading}>
        저장하기
      </button>
    </div>
  );
};

export default MasterConfigEditor;
