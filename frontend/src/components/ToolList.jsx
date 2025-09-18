import React from 'react';

const ToolList = ({ tools, onSync, diffs, loading }) => {
  if (!tools.length) {
    return <p className="empty">검색된 도구가 없습니다. 도구 재검색을 실행해 주세요.</p>;
  }

  return (
    <table className="tool-table">
      <thead>
        <tr>
          <th>도구명</th>
          <th>구성 파일</th>
          <th>차이</th>
          <th>동작</th>
        </tr>
      </thead>
      <tbody>
        {tools.map((tool) => (
          <tr key={tool.name}>
            <td>{tool.name}</td>
            <td><code>{tool.config_path}</code></td>
            <td>
              {diffs[tool.name] ? (
                <details>
                  <summary>차이 보기</summary>
                  <pre>{JSON.stringify(diffs[tool.name], null, 2)}</pre>
                </details>
              ) : (
                <span className="pill pill-success">동일</span>
              )}
            </td>
            <td>
              <button onClick={() => onSync(tool.name)} disabled={loading}>
                동기화
              </button>
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  );
};

export default ToolList;
