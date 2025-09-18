import React, { useEffect, useMemo, useState } from 'react';
import {
  fetchMasterConfig,
  fetchRecommendedServers,
  fetchSyncHistory,
  fetchTools,
  importRecommendedServer,
  rescanTools,
  syncTools,
  updateMasterConfig
} from './services/api.js';
import ToolList from './components/ToolList.jsx';
import MasterConfigEditor from './components/MasterConfigEditor.jsx';
import SyncHistory from './components/SyncHistory.jsx';
import RecommendedServerList from './components/RecommendedServerList.jsx';

const App = () => {
  const [tools, setTools] = useState([]);
  const [masterConfig, setMasterConfig] = useState(null);
  const [masterConfigDraft, setMasterConfigDraft] = useState('');
  const [history, setHistory] = useState([]);
  const [recommendedServers, setRecommendedServers] = useState([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [successMessage, setSuccessMessage] = useState('');

  useEffect(() => {
    bootstrap();
  }, []);

  const bootstrap = async () => {
    try {
      setLoading(true);
      const [toolList, master, historyItems, recommended] = await Promise.all([
        fetchTools(),
        fetchMasterConfig(),
        fetchSyncHistory(),
        fetchRecommendedServers()
      ]);
      setTools(toolList);
      setMasterConfig(master);
      setMasterConfigDraft(JSON.stringify(master.settings, null, 2));
      setHistory(historyItems);
      setRecommendedServers(recommended);
    } catch (err) {
      console.error(err);
      setError('초기 데이터를 불러오는데 실패했습니다. 백엔드 서버가 실행 중인지 확인해주세요.');
    } finally {
      setLoading(false);
    }
  };

  const handleRescan = async () => {
    setError('');
    setSuccessMessage('');
    try {
      setLoading(true);
      const scanned = await rescanTools();
      setTools(scanned);
      setSuccessMessage(`${scanned.length}개의 도구 구성을 다시 불러왔습니다.`);
    } catch (err) {
      console.error(err);
      setError('도구 재검색에 실패했습니다.');
    } finally {
      setLoading(false);
    }
  };

  const handleSync = async (toolName) => {
    setError('');
    setSuccessMessage('');
    try {
      setLoading(true);
      const summaries = await syncTools(toolName);
      setSuccessMessage('동기화가 완료되었습니다.');
      setHistory(await fetchSyncHistory());
      if (!toolName) {
        const refreshed = await fetchTools();
        setTools(refreshed);
      } else {
        setTools((prev) =>
          prev.map((tool) =>
            tool.name === toolName
              ? { ...tool, settings: masterConfig.settings }
              : tool
          )
        );
      }
      return summaries;
    } catch (err) {
      console.error(err);
      setError('동기화 중 오류가 발생했습니다.');
      throw err;
    } finally {
      setLoading(false);
    }
  };

  const handleMasterConfigSave = async () => {
    setError('');
    setSuccessMessage('');
    try {
      const parsed = JSON.parse(masterConfigDraft);
      const updated = await updateMasterConfig(parsed);
      setMasterConfig(updated);
      setMasterConfigDraft(JSON.stringify(updated.settings, null, 2));
      setSuccessMessage('마스터 구성을 저장했습니다.');
    } catch (err) {
      console.error(err);
      setError('마스터 구성 저장에 실패했습니다. JSON 형식을 확인해주세요.');
    }
  };

  const handleImportRecommended = async (serverId, enabled) => {
    setError('');
    setSuccessMessage('');
    try {
      setLoading(true);
      const updated = await importRecommendedServer(serverId, enabled);
      setMasterConfig(updated);
      setMasterConfigDraft(JSON.stringify(updated.settings, null, 2));
      setSuccessMessage('추천 서버를 마스터 구성에 추가했습니다.');
    } catch (err) {
      console.error(err);
      setError('추천 서버를 추가하는 데 실패했습니다.');
    } finally {
      setLoading(false);
    }
  };

  const diffByTool = useMemo(() => {
    if (!masterConfig) return {};
    return tools.reduce((acc, tool) => {
      acc[tool.name] = computeConfigDiff(masterConfig.settings, tool.settings);
      return acc;
    }, {});
  }, [masterConfig, tools]);

  const masterServerIds = useMemo(() => {
    if (!masterConfig) return new Set();
    return new Set(masterConfig.settings.servers.map((server) => server.id));
  }, [masterConfig]);

  return (
    <div className="app">
      <header>
        <h1>MCP Sync Service</h1>
        <p>AI CLI 도구의 MCP 구성을 자동으로 동기화합니다.</p>
      </header>

      <main>
        <section className="panel">
          <h2>마스터 구성</h2>
          <MasterConfigEditor
            loading={loading}
            draft={masterConfigDraft}
            onChange={setMasterConfigDraft}
            onSave={handleMasterConfigSave}
          />
        </section>

        <section className="panel">
          <h2>추천 MCP 서버</h2>
          <p className="panel-subtitle">문서에서 제안하는 주요 서버를 빠르게 추가할 수 있습니다.</p>
          <RecommendedServerList
            servers={recommendedServers}
            installedServerIds={masterServerIds}
            onImport={handleImportRecommended}
            loading={loading}
          />
        </section>

        <section className="panel">
          <div className="panel-header">
            <h2>설치된 도구</h2>
            <div className="panel-actions">
              <button onClick={handleRescan} disabled={loading}>
                도구 재검색
              </button>
              <button onClick={() => handleSync()} disabled={loading}>
                전체 동기화
              </button>
            </div>
          </div>
          <ToolList tools={tools} onSync={handleSync} diffs={diffByTool} loading={loading} />
        </section>

        <section className="panel">
          <h2>최근 동기화 내역</h2>
          <SyncHistory history={history} />
        </section>
      </main>

      {(error || successMessage) && (
        <div className={`toast ${error ? 'error' : 'success'}`}>
          {error || successMessage}
        </div>
      )}
    </div>
  );
};

function computeConfigDiff(master, tool) {
  try {
    const masterServers = JSON.stringify(master.servers);
    const toolServers = JSON.stringify(tool.servers);
    if (masterServers === toolServers) {
      return null;
    }
    return {
      master: master.servers,
      tool: tool.servers
    };
  } catch (err) {
    console.error('Failed to compute diff', err);
    return null;
  }
}

export default App;
