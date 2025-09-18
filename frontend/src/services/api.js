const jsonHeaders = {
  'Content-Type': 'application/json'
};

async function handleResponse(response) {
  if (!response.ok) {
    const error = await response.json().catch(() => ({}));
    throw new Error(error.error || response.statusText);
  }
  return response.json();
}

export async function fetchTools() {
  const response = await fetch('/api/tools');
  return handleResponse(response);
}

export async function rescanTools() {
  const response = await fetch('/api/tools/rescan', {
    method: 'POST'
  });
  return handleResponse(response);
}

export async function fetchMasterConfig() {
  const response = await fetch('/api/config/master');
  return handleResponse(response);
}

export async function fetchRecommendedServers() {
  const response = await fetch('/api/config/recommended');
  return handleResponse(response);
}

export async function updateMasterConfig(settings) {
  const response = await fetch('/api/config/master', {
    method: 'POST',
    headers: jsonHeaders,
    body: JSON.stringify({ settings })
  });
  return handleResponse(response);
}

export async function importRecommendedServer(serverId, enabled) {
  const response = await fetch('/api/config/master/import', {
    method: 'POST',
    headers: jsonHeaders,
    body: JSON.stringify({ server_id: serverId, enabled })
  });
  return handleResponse(response);
}

export async function syncTools(tool) {
  const response = await fetch('/api/sync', {
    method: 'POST',
    headers: jsonHeaders,
    body: JSON.stringify({ tool: tool ?? null })
  });
  return handleResponse(response);
}

export async function fetchSyncHistory() {
  const response = await fetch('/api/sync/history');
  return handleResponse(response);
}
