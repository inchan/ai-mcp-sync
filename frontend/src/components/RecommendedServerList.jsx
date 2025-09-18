import React from 'react';

const RecommendedServerList = ({ servers, installedServerIds, onImport, loading }) => {
  if (!servers.length) {
    return <p className="empty">추천 서버 데이터가 없습니다.</p>;
  }

  return (
    <div className="recommended-list">
      {servers.map((server) => {
        const installed = installedServerIds.has(server.id);
        return (
          <article key={server.id} className="recommended-card">
            <header className="recommended-card-header">
              <div>
                <h3>{server.name}</h3>
                {server.category && <span className="badge">{server.category}</span>}
              </div>
              <span className={`pill ${server.default_enabled ? 'pill-success' : 'pill-neutral'}`}>
                기본 {server.default_enabled ? '활성화' : '비활성화'}
              </span>
            </header>

            {server.description && <p className="recommended-description">{server.description}</p>}

            <dl className="recommended-meta">
              <div>
                <dt>엔드포인트</dt>
                <dd>
                  <code>{server.endpoint}</code>
                </dd>
              </div>
              <div>
                <dt>API 키</dt>
                <dd>{server.api_key_required ? '필요' : '불필요'}</dd>
              </div>
              {server.homepage && (
                <div>
                  <dt>홈페이지</dt>
                  <dd>
                    <a href={server.homepage} target="_blank" rel="noreferrer noopener">
                      공식 문서
                    </a>
                  </dd>
                </div>
              )}
            </dl>

            <div className="recommended-actions">
              <button
                onClick={() => onImport(server.id, server.default_enabled)}
                disabled={loading || installed}
              >
                {installed ? '이미 추가됨' : '마스터에 추가'}
              </button>
              {server.homepage && (
                <a
                  className="link-button"
                  href={server.homepage}
                  target="_blank"
                  rel="noreferrer noopener"
                >
                  문서 보기
                </a>
              )}
            </div>

            {server.api_key_required && (
              <p className="recommended-hint">추가 후 마스터 구성에서 API 키를 입력해야 합니다.</p>
            )}
          </article>
        );
      })}
    </div>
  );
};

export default RecommendedServerList;
