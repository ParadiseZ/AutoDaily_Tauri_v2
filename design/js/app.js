// Mock Data
const devices = [
    { id: 'dev1', name: 'Pixel 6 Pro', ip: '192.168.1.101', status: 'busy', currentScript: 'Game A - Daily', currentTask: 'Sign-in', queue: ['Dungeon Run', 'Mail Collect'], enabled: true, checked: false },
    { id: 'dev2', name: 'Emulator-5554', ip: 'localhost:5555', status: 'online', currentScript: null, currentTask: null, queue: [], enabled: true, checked: false },
    { id: 'dev3', name: 'Xiaomi 13', ip: '192.168.1.105', status: 'busy', currentScript: 'Game B - Event', currentTask: 'Boss Battle', queue: ['Reward Claim'], enabled: false, checked: false },
    { id: 'dev4', name: 'Samsung S23', ip: '192.168.1.108', status: 'offline', currentScript: null, currentTask: null, queue: [], enabled: true, checked: false }
];

const localScripts = [
    { id: 's1', name: 'Game A - Daily Routine', version: '1.2.0', author: 'User', description: 'Complete daily tasks for Game A' },
    { id: 's2', name: 'Game B - Event Farm', version: '2.0.1', author: 'User', description: 'Auto farm event currency' },
    { id: 's3', name: 'System - Clean Cache', version: '1.0.0', author: 'Admin', description: 'Clear app caches' }
];

const communityScripts = [
    { id: 'c1', name: 'Genshin Daily', downloads: '12k', rating: 4.8, author: 'AutoGod' },
    { id: 'c2', name: 'Arknights Base', downloads: '8.5k', rating: 4.9, author: 'DrRhodes' },
    { id: 'c3', name: 'Blue Archive Cafe', downloads: '5k', rating: 4.7, author: 'Sensei' }
];

// State
let activeView = 'dashboard';

// Initialization
document.addEventListener('DOMContentLoaded', () => {
    renderView();
    startLogSimulation();
});

// Navigation
function switchView(viewId) {
    activeView = viewId;
    document.querySelectorAll('.nav-item').forEach(el => el.classList.remove('active'));
    document.getElementById(`nav-${viewId}`).classList.add('active');
    renderView();
}

// Rendering
function renderView() {
    const contentArea = document.getElementById('content-area');
    const pageTitle = document.getElementById('page-title');

    contentArea.innerHTML = '';

    switch (activeView) {
        case 'dashboard':
            pageTitle.innerText = '多设备日志监控';
            renderDashboard(contentArea);
            break;
        case 'devices':
            pageTitle.innerText = '设备管理';
            renderDevices(contentArea);
            break;
        case 'scripts':
            pageTitle.innerText = '本地脚本';
            renderScripts(contentArea);
            break;
        case 'community':
            pageTitle.innerText = '脚本市场';
            renderCommunity(contentArea);
            break;
        case 'about':
            pageTitle.innerText = '关于';
            renderAbout(contentArea);
            break;
        default:
            contentArea.innerHTML = '<p>View not implemented</p>';
    }
}

// --- Dashboard (Log Columns) ---
function renderDashboard(container) {
    // Toolbar
    const toolbar = document.createElement('div');
    toolbar.className = 'log-toolbar';
    toolbar.innerHTML = `
        <span style="font-size:0.85rem; color:var(--text-secondary)">日志级别:</span>
        <select style="background:var(--card-bg); color:var(--text-primary); border:1px solid var(--border-color); padding:0.25rem 0.5rem; border-radius:0.25rem;">
            <option value="all">ALL</option>
            <option value="info" selected>INFO</option>
            <option value="warn">WARN</option>
            <option value="error">ERROR</option>
        </select>
        <button class="btn btn-sm btn-secondary">清除日志</button>
    `;
    container.appendChild(toolbar);

    const wrapper = document.createElement('div');
    wrapper.className = 'log-columns-container';

    // Only show active devices or all? Let's show active/busy devices for logs
    const activeDevices = devices.filter(d => d.status !== 'offline');

    if (activeDevices.length === 0) {
        wrapper.innerHTML = '<div style="padding:2rem; color:var(--text-secondary)">暂无运行中的设备</div>';
    } else {
        activeDevices.forEach(dev => {
            const col = document.createElement('div');
            col.className = 'log-column';
            col.innerHTML = `
                <div class="log-header">
                    <span style="font-weight:600; font-size:0.9rem">📱 ${dev.name}</span>
                    <span class="badge">${dev.status}</span>
                </div>
                <div class="log-body" id="log-${dev.id}">
                    <!-- Logs will be injected here -->
                </div>
            `;
            wrapper.appendChild(col);
        });
    }
    container.appendChild(wrapper);
}

// --- Device Manager ---
function renderDevices(container) {
    const wrapper = document.createElement('div');
    wrapper.className = 'device-columns-container';

    devices.forEach(dev => {
        const col = document.createElement('div');
        col.className = 'device-column';

        // Status Class
        const statusClass = dev.status === 'online' ? 'status-online' : (dev.status === 'busy' ? 'status-busy' : 'status-offline');

        // Queue HTML
        let queueHtml = '';
        if (dev.status !== 'offline') {
            queueHtml = `
                <div class="queue-section">
                    <div class="queue-title">
                        <span>任务队列</span>
                        <button class="btn btn-sm btn-primary" onclick="alert('Quick Add Script to ${dev.name}')">+</button>
                    </div>
                    <div class="queue-list">
                        ${dev.currentScript ? `
                        <div class="queue-item active">
                            <div>
                                <div style="font-weight:600">▶ ${dev.currentScript}</div>
                                <div style="color:var(--text-secondary); font-size:0.75rem">正在执行: ${dev.currentTask}</div>
                            </div>
                        </div>` : '<div style="color:var(--text-secondary); font-size:0.8rem; padding:0.5rem">空闲</div>'}
                        
                        ${dev.queue.map(q => `
                        <div class="queue-item">
                            <span>⌛ ${q}</span>
                        </div>`).join('')}
                    </div>
                </div>
            `;
        } else {
            queueHtml = `<div style="padding:1rem; color:var(--text-secondary); text-align:center">设备已离线</div>`;
        }

        col.innerHTML = `
            <div class="device-header">
                <input type="checkbox" class="custom-checkbox" ${dev.checked ? 'checked' : ''} onchange="toggleDeviceCheck('${dev.id}', this.checked)">
                <div style="flex:1; display:flex; align-items:center; gap:0.5rem;">
                    <span class="device-status ${statusClass}"></span>
                    <span style="font-weight:600; white-space:nowrap; overflow:hidden; text-overflow:ellipsis;">${dev.name}</span>
                </div>
                <label class="toggle-switch">
                    <input type="checkbox" ${dev.enabled ? 'checked' : ''} onchange="toggleDeviceEnable('${dev.id}', this.checked)">
                    <span class="slider"></span>
                </label>
            </div>
            <div class="device-body">
                <div class="device-controls">
                    ${dev.status === 'busy' ? '<button class="btn btn-sm btn-danger" style="flex:1">停止</button>' : ''}
                    ${dev.status === 'online' ? '<button class="btn btn-sm btn-primary" style="flex:1">启动</button>' : ''}
                    <button class="btn btn-sm btn-secondary" style="flex:1">详情</button>
                </div>
                <div style="font-size:0.8rem; color:var(--text-secondary); margin-bottom:1rem;">
                    IP: ${dev.ip}
                </div>
                ${queueHtml}
            </div>
        `;
        wrapper.appendChild(col);
    });

    // FAB Run Button
    const fab = document.createElement('button');
    fab.className = 'fab-run';
    fab.innerHTML = '▶';
    fab.title = '运行选中设备';
    fab.onclick = () => {
        const selected = devices.filter(d => d.checked && d.enabled);
        if (selected.length === 0) {
            alert('请先勾选并启用至少一个设备');
        } else {
            alert(`正在启动 ${selected.length} 个设备...`);
        }
    };
    wrapper.appendChild(fab);

    container.appendChild(wrapper);
}

// --- Local Scripts ---
function renderScripts(container) {
    const list = document.createElement('div');
    list.className = 'script-list';

    // Toolbar
    const toolbar = document.createElement('div');
    toolbar.style.marginBottom = '1rem';
    toolbar.innerHTML = `<button class="btn btn-primary">新建脚本</button>`;
    container.appendChild(toolbar);

    localScripts.forEach(script => {
        const item = document.createElement('div');
        item.className = 'script-item';
        item.innerHTML = `
            <div style="display:flex; align-items:center">
                <div class="script-icon">📜</div>
                <div>
                    <div style="font-weight:600; font-size:1rem">${script.name}</div>
                    <div style="color:var(--text-secondary); font-size:0.85rem">${script.description}</div>
                    <div style="margin-top:0.25rem">
                        <span class="badge">v${script.version}</span>
                        <span class="badge" style="margin-left:0.5rem">${script.author}</span>
                    </div>
                </div>
            </div>
            <div style="display:flex; gap:0.5rem">
                <button class="btn btn-secondary">编辑</button>
                <button class="btn btn-primary">运行</button>
            </div>
        `;
        list.appendChild(item);
    });

    container.appendChild(list);
}

// --- Community ---
function renderCommunity(container) {
    const grid = document.createElement('div');
    grid.className = 'community-grid';

    communityScripts.forEach(script => {
        const card = document.createElement('div');
        card.className = 'store-card';
        card.innerHTML = `
            <div class="store-cover">🎮</div>
            <div class="store-body">
                <div style="font-weight:600; margin-bottom:0.5rem">${script.name}</div>
                <div style="display:flex; justify-content:space-between; font-size:0.8rem; color:var(--text-secondary)">
                    <span>⭐ ${script.rating}</span>
                    <span>📥 ${script.downloads}</span>
                </div>
                <div style="margin-top:0.5rem; font-size:0.8rem">by ${script.author}</div>
            </div>
            <div class="store-footer">
                <span style="font-weight:600; color:var(--success-color)">Free</span>
                <button class="btn btn-sm btn-primary">下载</button>
            </div>
        `;
        grid.appendChild(card);
    });

    container.appendChild(grid);
}

// --- About ---
function renderAbout(container) {
    container.innerHTML = `
        <div class="card" style="max-width: 600px; margin: 0 auto; text-align: center;">
            <h2 style="margin-bottom: 1rem;">AutoDaily</h2>
            <p style="color: var(--text-secondary); margin-bottom: 2rem;">
                一个强大的多设备自动化脚本管理工具。<br>
                旨在简化游戏日常任务与设备群控流程。
            </p>
            <div style="display: flex; flex-direction: column; gap: 1rem; align-items: center;">
                <div style="display: flex; gap: 1rem;">
                    <button class="btn btn-secondary">GitHub</button>
                    <button class="btn btn-secondary">文档</button>
                </div>
                <div style="margin-top: 2rem; padding-top: 1rem; border-top: 1px solid var(--border-color); width: 100%;">
                    <p style="font-size: 0.9rem; color: var(--text-secondary);">联系开发者</p>
                    <p style="font-weight: 600; margin-top: 0.5rem;">contact@autodaily.dev</p>
                </div>
            </div>
        </div>
    `;
}

// --- Log Simulation ---
function startLogSimulation() {
    setInterval(() => {
        if (activeView !== 'dashboard') return;

        const activeDevices = devices.filter(d => d.status !== 'offline');
        activeDevices.forEach(dev => {
            const logBody = document.getElementById(`log-${dev.id}`);
            if (logBody) {
                const now = new Date().toLocaleTimeString();
                const msgs = [
                    { type: 'info', text: `Executing step: ${Math.floor(Math.random() * 100)}` },
                    { type: 'info', text: 'Image match confidence: 0.98' },
                    { type: 'warn', text: 'Network delay detected (120ms)' },
                    { type: 'info', text: 'Task completed, moving to next' }
                ];
                const msg = msgs[Math.floor(Math.random() * msgs.length)];

                const line = document.createElement('div');
                line.className = 'log-line';
                line.innerHTML = `<span class="log-time">[${now}]</span><span class="log-level-${msg.type}">[${msg.type.toUpperCase()}]</span> ${msg.text}`;

                logBody.appendChild(line);
                logBody.scrollTop = logBody.scrollHeight;

                // Keep log size manageable
                if (logBody.children.length > 50) {
                    logBody.removeChild(logBody.firstChild);
                }
            }
        });
    }, 1000);
}

// Helper functions for device state
window.toggleDeviceCheck = (id, checked) => {
    const dev = devices.find(d => d.id === id);
    if (dev) dev.checked = checked;
};

window.toggleDeviceEnable = (id, enabled) => {
    const dev = devices.find(d => d.id === id);
    if (dev) dev.enabled = enabled;
};
