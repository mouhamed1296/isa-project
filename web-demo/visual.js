// MA-ISA Visual State Tree - Mind Map Visualization with Animations
import init, { WasmAxisAccumulator } from './pkg/isa_ffi.js';

// State management
const state = {
    wasmInitialized: false,
    tamperMode: false,
    dimensions: [],
    accumulators: [],
    referenceAccumulators: [],
    eventLog: [],
    totalEvents: 0,
    canvas: null,
    ctx: null,
    nodes: [],
    animations: []
};

// Configuration
const config = {
    dimensions: [
        { id: 0, name: "Sensor Integrity", color: "#4ade80" },
        { id: 1, name: "Network Integrity", color: "#60a5fa" },
        { id: 2, name: "Firmware Integrity", color: "#f472b6" },
        { id: 3, name: "Power Integrity", color: "#fbbf24" }
    ]
};

// Node class for mind map
class Node {
    constructor(x, y, type, label, data = {}) {
        this.x = x;
        this.y = y;
        this.type = type; // 'root', 'dimension', 'event'
        this.label = label;
        this.data = data;
        this.radius = type === 'root' ? 60 : type === 'dimension' ? 45 : 25;
        this.color = data.color || '#667eea';
        this.children = [];
        this.parent = null;
        this.divergence = 0;
        this.pulsePhase = 0;
    }

    draw(ctx) {
        // Draw connection to parent
        if (this.parent) {
            ctx.strokeStyle = this.divergence > 0 ? '#f87171' : 'rgba(255, 255, 255, 0.3)';
            ctx.lineWidth = this.divergence > 0 ? 3 : 2;
            ctx.beginPath();
            ctx.moveTo(this.parent.x, this.parent.y);
            ctx.lineTo(this.x, this.y);
            ctx.stroke();

            // Draw divergence indicator on line
            if (this.divergence > 0) {
                const midX = (this.parent.x + this.x) / 2;
                const midY = (this.parent.y + this.y) / 2;
                ctx.fillStyle = '#f87171';
                ctx.font = 'bold 12px Arial';
                ctx.fillText(`⚠️ ${this.divergence}`, midX + 10, midY - 10);
            }
        }

        // Pulse effect for divergence
        if (this.divergence > 0) {
            this.pulsePhase += 0.1;
            const pulseScale = 1 + Math.sin(this.pulsePhase) * 0.2;
            
            // Outer pulse ring
            ctx.strokeStyle = '#f87171';
            ctx.lineWidth = 3;
            ctx.globalAlpha = 0.5 + Math.sin(this.pulsePhase) * 0.3;
            ctx.beginPath();
            ctx.arc(this.x, this.y, this.radius * pulseScale, 0, Math.PI * 2);
            ctx.stroke();
            ctx.globalAlpha = 1;
        }

        // Draw node circle
        const gradient = ctx.createRadialGradient(this.x, this.y, 0, this.x, this.y, this.radius);
        gradient.addColorStop(0, this.color);
        gradient.addColorStop(1, this.adjustColor(this.color, -30));
        
        ctx.fillStyle = gradient;
        ctx.beginPath();
        ctx.arc(this.x, this.y, this.radius, 0, Math.PI * 2);
        ctx.fill();

        // Border
        ctx.strokeStyle = this.divergence > 0 ? '#f87171' : 'rgba(255, 255, 255, 0.5)';
        ctx.lineWidth = this.divergence > 0 ? 4 : 2;
        ctx.stroke();

        // Label
        ctx.fillStyle = '#fff';
        ctx.font = `bold ${this.type === 'root' ? 16 : 12}px Arial`;
        ctx.textAlign = 'center';
        ctx.textBaseline = 'middle';
        
        // Multi-line text for long labels
        const words = this.label.split(' ');
        if (words.length > 2 && this.type !== 'event') {
            ctx.fillText(words.slice(0, 2).join(' '), this.x, this.y - 5);
            ctx.fillText(words.slice(2).join(' '), this.x, this.y + 10);
        } else {
            ctx.fillText(this.label, this.x, this.y);
        }

        // Event counter badge
        if (this.type === 'dimension' && this.data.eventCount > 0) {
            ctx.fillStyle = '#667eea';
            ctx.beginPath();
            ctx.arc(this.x + this.radius - 10, this.y - this.radius + 10, 12, 0, Math.PI * 2);
            ctx.fill();
            ctx.fillStyle = '#fff';
            ctx.font = 'bold 10px Arial';
            ctx.fillText(this.data.eventCount, this.x + this.radius - 10, this.y - this.radius + 10);
        }
    }

    adjustColor(color, amount) {
        const num = parseInt(color.replace('#', ''), 16);
        const r = Math.max(0, Math.min(255, (num >> 16) + amount));
        const g = Math.max(0, Math.min(255, ((num >> 8) & 0x00FF) + amount));
        const b = Math.max(0, Math.min(255, (num & 0x0000FF) + amount));
        return `#${((r << 16) | (g << 8) | b).toString(16).padStart(6, '0')}`;
    }
}

// Animation class for event accumulation
class AccumulationAnimation {
    constructor(fromNode, toNode, eventData) {
        this.fromNode = fromNode;
        this.toNode = toNode;
        this.progress = 0;
        this.speed = 0.02;
        this.eventData = eventData;
        this.particles = [];
        
        // Create particles
        for (let i = 0; i < 10; i++) {
            this.particles.push({
                offset: Math.random() * 20 - 10,
                phase: Math.random() * Math.PI * 2,
                size: Math.random() * 3 + 2
            });
        }
    }

    update() {
        this.progress += this.speed;
        return this.progress >= 1;
    }

    draw(ctx) {
        const x = this.fromNode.x + (this.toNode.x - this.fromNode.x) * this.progress;
        const y = this.fromNode.y + (this.toNode.y - this.fromNode.y) * this.progress;

        // Draw particles
        this.particles.forEach(particle => {
            const px = x + Math.cos(particle.phase + this.progress * 10) * particle.offset;
            const py = y + Math.sin(particle.phase + this.progress * 10) * particle.offset;
            
            ctx.fillStyle = `rgba(102, 126, 234, ${1 - this.progress})`;
            ctx.beginPath();
            ctx.arc(px, py, particle.size, 0, Math.PI * 2);
            ctx.fill();
        });

        // Draw main orb
        const gradient = ctx.createRadialGradient(x, y, 0, x, y, 15);
        gradient.addColorStop(0, 'rgba(102, 126, 234, 0.8)');
        gradient.addColorStop(1, 'rgba(118, 75, 162, 0.4)');
        
        ctx.fillStyle = gradient;
        ctx.beginPath();
        ctx.arc(x, y, 15, 0, Math.PI * 2);
        ctx.fill();

        // Glow effect
        ctx.strokeStyle = `rgba(102, 126, 234, ${0.5 - this.progress * 0.5})`;
        ctx.lineWidth = 3;
        ctx.beginPath();
        ctx.arc(x, y, 15 + this.progress * 10, 0, Math.PI * 2);
        ctx.stroke();
    }
}

// Initialize WASM
async function initializeWasm() {
    console.log('[VISUAL] Initializing WASM...');
    try {
        await init();
        
        // Create accumulators for each dimension
        for (const dim of config.dimensions) {
            const seed = new Uint8Array(32);
            crypto.getRandomValues(seed);
            
            const accumulator = new WasmAxisAccumulator(seed);
            const refAccumulator = new WasmAxisAccumulator(seed);
            
            state.accumulators.push(accumulator);
            state.referenceAccumulators.push(refAccumulator);
            
            state.dimensions.push({
                id: dim.id,
                name: dim.name,
                color: dim.color,
                state: bytesToHex(accumulator.state()),
                eventCount: 0,
                divergence: 0
            });
        }
        
        state.wasmInitialized = true;
        updateStatus();
        initializeMindMap();
        logEvent('SYSTEM', 'WASM initialized successfully');
        
        console.log('[VISUAL] WASM initialization complete');
    } catch (error) {
        console.error('[VISUAL] WASM initialization failed:', error);
        alert('Failed to initialize WASM: ' + error.message);
    }
}

// Initialize mind map
function initializeMindMap() {
    const canvas = document.getElementById('mindMapCanvas');
    const ctx = canvas.getContext('2d');
    
    // Set canvas size
    canvas.width = canvas.offsetWidth;
    canvas.height = canvas.offsetHeight;
    
    state.canvas = canvas;
    state.ctx = ctx;
    
    // Create root node
    const rootNode = new Node(
        canvas.width / 2,
        canvas.height / 2,
        'root',
        'MA-ISA Root',
        { color: '#667eea' }
    );
    state.nodes = [rootNode];
    
    // Create dimension nodes in a circle around root
    const angleStep = (Math.PI * 2) / config.dimensions.length;
    const radius = 200;
    
    config.dimensions.forEach((dim, index) => {
        const angle = angleStep * index - Math.PI / 2;
        const x = rootNode.x + Math.cos(angle) * radius;
        const y = rootNode.y + Math.sin(angle) * radius;
        
        const dimNode = new Node(x, y, 'dimension', dim.name, {
            color: dim.color,
            dimensionId: dim.id,
            eventCount: 0
        });
        dimNode.parent = rootNode;
        rootNode.children.push(dimNode);
        state.nodes.push(dimNode);
    });
    
    // Start render loop
    renderLoop();
}

// Render loop
function renderLoop() {
    const ctx = state.ctx;
    const canvas = state.canvas;
    
    // Clear canvas
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    
    // Draw all nodes
    state.nodes.forEach(node => node.draw(ctx));
    
    // Update and draw animations
    state.animations = state.animations.filter(anim => {
        anim.draw(ctx);
        return !anim.update();
    });
    
    requestAnimationFrame(renderLoop);
}

// Record event
window.recordEvent = function(dimensionId) {
    if (!state.wasmInitialized) {
        alert('WASM not initialized yet!');
        return;
    }
    
    const dimension = state.dimensions[dimensionId];
    const accumulator = state.accumulators[dimensionId];
    const refAccumulator = state.referenceAccumulators[dimensionId];
    
    // Generate event data
    const eventData = generateEventData(dimensionId);
    const eventBytes = new TextEncoder().encode(eventData);
    
    // Generate entropy
    const entropy = new Uint8Array(32);
    crypto.getRandomValues(entropy);
    
    const deltaT = BigInt(Date.now());
    
    // Prepare potentially tampered data
    let activeEventBytes = eventBytes;
    if (state.tamperMode) {
        const tamperedData = eventData + ' [TAMPERED]';
        activeEventBytes = new TextEncoder().encode(tamperedData);
        console.log('[VISUAL] Tampered event:', tamperedData);
    }
    
    // Accumulate on both accumulators
    accumulator.accumulate(activeEventBytes, entropy, deltaT);
    refAccumulator.accumulate(eventBytes, entropy, deltaT);
    
    // Update state
    const newState = accumulator.state();
    const refState = refAccumulator.state();
    dimension.state = bytesToHex(newState);
    dimension.eventCount++;
    dimension.divergence = calculateDivergence(newState, refState);
    
    state.totalEvents++;
    
    // Update mind map
    const dimNode = state.nodes.find(n => n.data.dimensionId === dimensionId);
    if (dimNode) {
        dimNode.data.eventCount = dimension.eventCount;
        dimNode.divergence = dimension.divergence;
        
        // Create animation
        const rootNode = state.nodes[0];
        const animation = new AccumulationAnimation(rootNode, dimNode, eventData);
        state.animations.push(animation);
    }
    
    // Log event
    logEvent(dimension.name, eventData, dimension.divergence);
    updateStatus();
    
    console.log('[VISUAL] Event recorded:', { dimensionId, eventData, divergence: dimension.divergence });
};

// Generate event data
function generateEventData(dimensionId) {
    switch (dimensionId) {
        case 0:
            return `temp=${(20 + Math.random() * 10).toFixed(1)}°C, humidity=${(40 + Math.random() * 30).toFixed(1)}%`;
        case 1:
            const events = ['ping', 'data_sync', 'heartbeat', 'config_update'];
            return `network_event=${events[Math.floor(Math.random() * events.length)]}`;
        case 2:
            return `firmware_update=v1.${Math.floor(Math.random() * 10)}.${Math.floor(Math.random() * 100)}`;
        case 3:
            return `battery_level=${(50 + Math.random() * 50).toFixed(0)}%`;
        default:
            return 'unknown_event';
    }
}

// Toggle tamper mode
window.toggleTamper = function() {
    state.tamperMode = !state.tamperMode;
    
    const btn = document.getElementById('tamperBtn');
    const statusDiv = document.getElementById('tamperStatus').querySelector('.status-value');
    
    if (state.tamperMode) {
        btn.textContent = '🔒 Disable Tamper Mode';
        btn.className = 'btn-success';
        statusDiv.textContent = 'Tamper Mode';
        statusDiv.className = 'status-value status-error';
        logEvent('SYSTEM', 'Tamper mode ENABLED', 0);
    } else {
        btn.textContent = '🔓 Enable Tamper Mode';
        btn.className = 'btn-danger';
        statusDiv.textContent = 'Normal';
        statusDiv.className = 'status-value status-ok';
        logEvent('SYSTEM', 'Tamper mode DISABLED', 0);
    }
};

// Open reconciliation modal
window.openReconciliation = function() {
    const modal = document.getElementById('reconciliationModal');
    const content = document.getElementById('reconciliationContent');
    
    // Build comparison grid
    let html = '<div class="comparison-grid">';
    
    state.dimensions.forEach((dim, index) => {
        const activeState = dim.state;
        const refState = bytesToHex(state.referenceAccumulators[index].state());
        const isDiverged = dim.divergence > 0;
        
        html += `
            <div class="comparison-panel">
                <h3>${dim.name}</h3>
                <div style="margin-bottom: 15px;">
                    <strong>Status:</strong> 
                    <span class="${isDiverged ? 'status-error' : 'status-ok'}">
                        ${isDiverged ? '⚠️ DIVERGED' : '✅ SYNCHRONIZED'}
                    </span>
                </div>
                <div>
                    <strong>Active State:</strong>
                    <div class="state-display">${formatState(activeState, refState)}</div>
                </div>
                <div>
                    <strong>Reference State:</strong>
                    <div class="state-display">${refState.substring(0, 64)}...</div>
                </div>
                <div>
                    <strong>Divergence:</strong> ${dim.divergence}
                </div>
                <div>
                    <strong>Events:</strong> ${dim.eventCount}
                </div>
            </div>
        `;
    });
    
    html += '</div>';
    
    // Add reconciliation actions
    html += `
        <div style="margin-top: 30px; padding: 20px; background: rgba(0,0,0,0.3); border-radius: 10px;">
            <h3 style="margin-bottom: 15px;">🔧 Reconciliation Actions</h3>
            <p style="margin-bottom: 15px;">When divergence is detected, you can:</p>
            <ul style="margin-left: 20px; line-height: 1.8;">
                <li><strong>Investigate:</strong> Review event logs to identify tampered events</li>
                <li><strong>Resync:</strong> Reset active accumulator to match reference state</li>
                <li><strong>Alert:</strong> Trigger security alerts for divergence threshold violations</li>
                <li><strong>Quarantine:</strong> Isolate affected dimension until reconciliation</li>
            </ul>
        </div>
    `;
    
    content.innerHTML = html;
    modal.classList.add('active');
};

// Close reconciliation modal
window.closeReconciliation = function() {
    document.getElementById('reconciliationModal').classList.remove('active');
};

// Format state with diff highlighting
function formatState(activeState, refState) {
    if (activeState === refState) {
        return activeState.substring(0, 64) + '...';
    }
    
    // Highlight differences
    let html = '';
    for (let i = 0; i < Math.min(64, activeState.length); i++) {
        if (activeState[i] !== refState[i]) {
            html += `<span class="diff-highlight">${activeState[i]}</span>`;
        } else {
            html += activeState[i];
        }
    }
    return html + '...';
}

// Reset system
window.resetSystem = async function() {
    if (!confirm('Reset all accumulators and clear event history?')) {
        return;
    }
    
    state.dimensions = [];
    state.accumulators = [];
    state.referenceAccumulators = [];
    state.eventLog = [];
    state.totalEvents = 0;
    state.animations = [];
    state.tamperMode = false;
    
    await initializeWasm();
    logEvent('SYSTEM', 'System reset complete', 0);
};

// Update status display
function updateStatus() {
    document.getElementById('wasmStatus').textContent = state.wasmInitialized ? '✅ Ready' : '⏳ Loading';
    document.getElementById('wasmStatus').className = state.wasmInitialized ? 'status-value status-ok' : 'status-value status-warning';
    
    document.getElementById('dimensionCount').textContent = state.dimensions.length;
    document.getElementById('eventCount').textContent = state.totalEvents;
    
    const maxDivergence = Math.max(...state.dimensions.map(d => d.divergence), 0);
    const divergenceEl = document.getElementById('divergenceStatus');
    
    if (maxDivergence === 0) {
        divergenceEl.textContent = 'None';
        divergenceEl.className = 'status-value status-ok';
    } else if (maxDivergence < 50000) {
        divergenceEl.textContent = `Low (${maxDivergence})`;
        divergenceEl.className = 'status-value status-warning';
    } else {
        divergenceEl.textContent = `High (${maxDivergence})`;
        divergenceEl.className = 'status-value status-error';
    }
}

// Log event
function logEvent(dimension, data, divergence) {
    const timestamp = new Date().toLocaleTimeString();
    const entry = { timestamp, dimension, data, divergence };
    
    state.eventLog.unshift(entry);
    if (state.eventLog.length > 20) {
        state.eventLog.pop();
    }
    
    // Update UI
    const logContainer = document.getElementById('eventLog');
    logContainer.innerHTML = state.eventLog.map(e => `
        <div class="event-entry">
            <span class="event-timestamp">${e.timestamp}</span> 
            <strong>${e.dimension}:</strong> ${e.data}
            ${e.divergence > 0 ? `<span class="status-error"> ⚠️ Δ${e.divergence}</span>` : ''}
        </div>
    `).join('');
}

// Utility functions
function bytesToHex(bytes) {
    return Array.from(bytes).map(b => b.toString(16).padStart(2, '0')).join('');
}

function calculateDivergence(state1Bytes, state2Bytes) {
    let state1 = BigInt('0x' + bytesToHex(state1Bytes));
    let state2 = BigInt('0x' + bytesToHex(state2Bytes));
    
    const maxValue = BigInt(2) ** BigInt(256);
    let diff = state1 > state2 ? state1 - state2 : state2 - state1;
    
    if (diff > maxValue / BigInt(2)) {
        diff = maxValue - diff;
    }
    
    return Number(diff % BigInt(100000));
}

// Initialize on load
document.addEventListener('DOMContentLoaded', async () => {
    console.log('[VISUAL] Starting visual demo...');
    await initializeWasm();
});
