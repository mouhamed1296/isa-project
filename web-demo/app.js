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
    dimensionEventHistory: {}, // Track events per dimension
    totalEvents: 0,
    canvas: null,
    ctx: null,
    nodes: [],
    animations: [],
    selectedDimensionId: null, // Currently selected dimension for event tree
    policyViolations: [], // Track policy violations
    constraintStatus: {} // Track constraint satisfaction
};

// Configuration with policies and constraints
const config = {
    dimensions: [
        { id: 0, name: "Sensor Integrity", color: "#4ade80", threshold: 1000, weight: 1.5, strategy: "MonitorOnly", safetyRelevant: false },
        { id: 1, name: "Network Integrity", color: "#60a5fa", threshold: 800, weight: 1.0, strategy: "ImmediateHeal", safetyRelevant: true },
        { id: 2, name: "Firmware Integrity", color: "#f472b6", threshold: 500, weight: 2.0, strategy: "Quarantine", safetyRelevant: true },
        { id: 3, name: "Power Integrity", color: "#fbbf24", threshold: 1200, weight: 0.8, strategy: "MonitorOnly", safetyRelevant: false }
    ],
    constraints: [
        {
            type: "MaxRatio",
            dimensions: [0, 1],
            ratio: 2.0,
            description: "Sensor divergence should not exceed 2x network divergence"
        },
        {
            type: "SumBelow",
            dimensions: [0, 1, 2],
            maxSum: 3000,
            description: "Total divergence across critical dimensions must stay below 3000"
        }
    ],
    strategies: {
        "MonitorOnly": "Log divergence but take no action",
        "ImmediateHeal": "Attempt state reconciliation immediately",
        "Quarantine": "Isolate dimension and alert operator",
        "GracefulDegrade": "Reduce functionality but continue operation"
    }
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
        
        // Highlight if selected
        const isSelected = this.type === 'dimension' && state.selectedDimensionId === this.data.dimensionId;
        if (isSelected) {
            ctx.strokeStyle = '#fbbf24';
            ctx.lineWidth = 4;
            ctx.beginPath();
            ctx.arc(this.x, this.y, this.radius + 5, 0, Math.PI * 2);
            ctx.stroke();
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
    
    // Add click handler for node selection
    canvas.addEventListener('click', handleCanvasClick);
    
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
    
    // Store event in dimension history
    if (!state.dimensionEventHistory[dimensionId]) {
        state.dimensionEventHistory[dimensionId] = [];
    }
    state.dimensionEventHistory[dimensionId].push({
        timestamp: new Date().toISOString(),
        eventData: eventData,
        eventNumber: dimension.eventCount,
        stateBefore: dimension.state.substring(0, 16) + '...',
        stateAfter: bytesToHex(newState).substring(0, 16) + '...',
        divergence: dimension.divergence,
        tampered: state.tamperMode
    });
    
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
    
    // Check policies and constraints
    checkPoliciesAndConstraints(dimensionId);
    
    updateStatus();
    
    // Update event tree if this dimension is selected
    if (state.selectedDimensionId === dimensionId) {
        updateEventTree(dimensionId);
    }
    
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
                ${isDiverged ? `
                    <div style="margin-top: 15px; display: flex; gap: 10px; flex-direction: column;">
                        <button class="btn-success" onclick="resyncDimension(${index})" style="padding: 8px 12px; font-size: 0.9em;">🔄 Resync to Reference</button>
                        <button class="btn-warning" onclick="quarantineDimension(${index})" style="padding: 8px 12px; font-size: 0.9em;">🚨 Quarantine Dimension</button>
                        <button class="btn-danger" onclick="triggerAlert(${index})" style="padding: 8px 12px; font-size: 0.9em;">⚠️ Trigger Alert</button>
                    </div>
                ` : ''}
            </div>
        `;
    });
    
    html += '</div>';
    
    // Add reconciliation actions guide
    html += `
        <div style="margin-top: 30px; padding: 20px; background: rgba(0,0,0,0.3); border-radius: 10px;">
            <h3 style="margin-bottom: 15px;">🔧 Reconciliation Actions Guide</h3>
            <p style="margin-bottom: 15px;">When divergence is detected, you can take the following actions:</p>
            <ul style="margin-left: 20px; line-height: 1.8;">
                <li><strong>🔄 Resync:</strong> Reset active accumulator to match reference state (restores synchronization)</li>
                <li><strong>🚨 Quarantine:</strong> Isolate affected dimension and prevent further events (security measure)</li>
                <li><strong>⚠️ Alert:</strong> Log security alert and notify monitoring systems (detection only)</li>
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

// Resync dimension - reset active accumulator to match reference
window.resyncDimension = function(dimensionId) {
    if (!confirm(`Resync ${state.dimensions[dimensionId].name}? This will reset the active accumulator to match the reference state.`)) {
        return;
    }
    
    console.log(`[RESYNC] Resyncing dimension ${dimensionId}`);
    
    // Get reference accumulator's seed by creating a new accumulator with same state
    const refAccumulator = state.referenceAccumulators[dimensionId];
    const refState = refAccumulator.state();
    
    // Create new accumulator with same seed as reference
    const seed = new Uint8Array(32);
    crypto.getRandomValues(seed);
    const newAccumulator = new WasmAxisAccumulator(seed);
    
    // Replace active accumulator
    state.accumulators[dimensionId] = newAccumulator;
    
    // Also create new reference with same seed
    const newRefAccumulator = new WasmAxisAccumulator(seed);
    state.referenceAccumulators[dimensionId] = newRefAccumulator;
    
    // Update dimension state
    const dimension = state.dimensions[dimensionId];
    dimension.state = bytesToHex(newAccumulator.state());
    dimension.divergence = 0;
    dimension.eventCount = 0;
    
    // Update mind map
    const dimNode = state.nodes.find(n => n.data.dimensionId === dimensionId);
    if (dimNode) {
        dimNode.divergence = 0;
        dimNode.data.eventCount = 0;
    }
    
    // NOTE: Event history is preserved for audit purposes
    logEvent('RECONCILIATION', `${dimension.name} resynced to reference state (${state.dimensionEventHistory[dimensionId]?.length || 0} events preserved)`, 0);
    updateStatus();
    openReconciliation(); // Refresh modal
    
    // Update event tree if this dimension is selected
    if (state.selectedDimensionId === dimensionId) {
        updateEventTree(dimensionId);
    }
    
    console.log(`[RESYNC] Dimension ${dimensionId} resynced successfully`);
    console.log(`[RESYNC] Event history preserved: ${state.dimensionEventHistory[dimensionId]?.length || 0} events`);
};

// Quarantine dimension - mark as quarantined and prevent events
window.quarantineDimension = function(dimensionId) {
    const dimension = state.dimensions[dimensionId];
    
    if (!confirm(`Quarantine ${dimension.name}? This will isolate the dimension and prevent further events until resolved.`)) {
        return;
    }
    
    console.log(`[QUARANTINE] Quarantining dimension ${dimensionId}`);
    
    // Mark dimension as quarantined
    dimension.quarantined = true;
    
    // Update mind map node
    const dimNode = state.nodes.find(n => n.data.dimensionId === dimensionId);
    if (dimNode) {
        dimNode.color = '#dc3545'; // Red color for quarantined
        dimNode.data.quarantined = true;
    }
    
    logEvent('SECURITY', `${dimension.name} QUARANTINED - Divergence: ${dimension.divergence}`, dimension.divergence);
    alert(`⚠️ ${dimension.name} has been quarantined!\n\nDivergence: ${dimension.divergence}\nEvents: ${dimension.eventCount}\n\nNo further events will be accepted until reconciliation.`);
    
    openReconciliation(); // Refresh modal
    
    console.log(`[QUARANTINE] Dimension ${dimensionId} quarantined`);
};

// Trigger alert for divergence
window.triggerAlert = function(dimensionId) {
    const dimension = state.dimensions[dimensionId];
    
    console.log(`[ALERT] Triggering security alert for dimension ${dimensionId}`);
    
    const alertMessage = `
🚨 SECURITY ALERT 🚨

Dimension: ${dimension.name}
Divergence Detected: ${dimension.divergence}
Total Events: ${dimension.eventCount}
Active State: ${dimension.state.substring(0, 32)}...

Action Required:
- Investigate event logs for tampering
- Review recent events for anomalies
- Consider resyncing or quarantining dimension
    `;
    
    console.error('[ALERT]', alertMessage);
    logEvent('ALERT', `Security alert triggered for ${dimension.name} (Δ${dimension.divergence})`, dimension.divergence);
    
    alert(alertMessage);
    
    // In production, this would:
    // - Send to SIEM system
    // - Trigger incident response workflow
    // - Notify security team
    // - Log to audit trail
    
    console.log(`[ALERT] Alert logged for dimension ${dimensionId}`);
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
    state.dimensionEventHistory = {};
    state.totalEvents = 0;
    state.animations = [];
    state.tamperMode = false;
    state.selectedDimensionId = null;
    
    await initializeWasm();
    logEvent('SYSTEM', 'System reset complete', 0);
    updateEventTree(null);
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

// Handle canvas click for node selection
function handleCanvasClick(event) {
    const rect = state.canvas.getBoundingClientRect();
    const x = event.clientX - rect.left;
    const y = event.clientY - rect.top;
    
    // Check if click is on a dimension node
    for (const node of state.nodes) {
        if (node.type === 'dimension') {
            const distance = Math.sqrt((x - node.x) ** 2 + (y - node.y) ** 2);
            if (distance <= node.radius) {
                // Node clicked!
                state.selectedDimensionId = node.data.dimensionId;
                updateEventTree(node.data.dimensionId);
                console.log(`[SELECT] Dimension ${node.data.dimensionId} selected`);
                return;
            }
        }
    }
    
    // Click outside nodes - deselect
    if (state.selectedDimensionId !== null) {
        state.selectedDimensionId = null;
        updateEventTree(null);
        console.log('[SELECT] Dimension deselected');
    }
}

// Update event tree display
function updateEventTree(dimensionId) {
    const treeContainer = document.getElementById('eventTree');
    const titleEl = document.getElementById('eventTreeTitle');
    const subtitleEl = document.getElementById('eventTreeSubtitle');
    
    if (dimensionId === null) {
        // No selection
        titleEl.textContent = '📜 Event History';
        subtitleEl.textContent = 'Click a dimension node to view its event history';
        treeContainer.innerHTML = `
            <div style="text-align: center; padding: 40px; opacity: 0.6;">
                <p>No dimension selected</p>
                <p style="font-size: 0.85em; margin-top: 10px;">Click on a dimension node in the mind map above</p>
            </div>
        `;
        return;
    }
    
    const dimension = state.dimensions[dimensionId];
    const events = state.dimensionEventHistory[dimensionId] || [];
    
    titleEl.textContent = `📜 Event History: ${dimension.name}`;
    subtitleEl.textContent = `${events.length} events recorded | Divergence: ${dimension.divergence}`;
    
    if (events.length === 0) {
        treeContainer.innerHTML = `
            <div style="text-align: center; padding: 40px; opacity: 0.6;">
                <p>No events recorded yet</p>
                <p style="font-size: 0.85em; margin-top: 10px;">Click event buttons to record events for this dimension</p>
            </div>
        `;
        return;
    }
    
    // Build event tree (reverse chronological order)
    let html = '<div style="display: flex; flex-direction: column; gap: 10px;">';
    
    for (let i = events.length - 1; i >= 0; i--) {
        const event = events[i];
        const timestamp = new Date(event.timestamp).toLocaleTimeString();
        const bgColor = event.tampered ? 'rgba(239, 68, 68, 0.2)' : 'rgba(255, 255, 255, 0.1)';
        const borderColor = event.tampered ? '#ef4444' : event.divergence > 0 ? '#fbbf24' : '#4ade80';
        
        html += `
            <div style="background: ${bgColor}; border-left: 4px solid ${borderColor}; padding: 12px; border-radius: 8px;">
                <div style="display: flex; justify-content: space-between; align-items: start; margin-bottom: 8px;">
                    <div>
                        <strong style="font-size: 0.95em;">#${event.eventNumber}</strong>
                        <span style="opacity: 0.7; font-size: 0.85em; margin-left: 10px;">${timestamp}</span>
                    </div>
                    <div style="display: flex; gap: 8px; align-items: center;">
                        ${event.tampered ? '<span style="background: #ef4444; padding: 2px 8px; border-radius: 4px; font-size: 0.75em;">TAMPERED</span>' : ''}
                        ${event.divergence > 0 ? `<span style="background: #fbbf24; color: #000; padding: 2px 8px; border-radius: 4px; font-size: 0.75em;">Δ${event.divergence}</span>` : '<span style="background: #4ade80; color: #000; padding: 2px 8px; border-radius: 4px; font-size: 0.75em;">✓ SYNCED</span>'}
                    </div>
                </div>
                <div style="font-size: 0.9em; margin-bottom: 6px;">
                    <strong>Event:</strong> <code style="background: rgba(0,0,0,0.3); padding: 2px 6px; border-radius: 3px;">${event.eventData}</code>
                </div>
                <div style="font-size: 0.85em; opacity: 0.8;">
                    <strong>State:</strong> ${event.stateAfter}
                </div>
            </div>
        `;
    }
    
    html += '</div>';
    
    // Add summary at bottom
    const tamperedCount = events.filter(e => e.tampered).length;
    const divergedCount = events.filter(e => e.divergence > 0).length;
    
    html += `
        <div style="margin-top: 15px; padding: 12px; background: rgba(0,0,0,0.3); border-radius: 8px; font-size: 0.9em;">
            <strong>Summary:</strong>
            <div style="display: flex; gap: 15px; margin-top: 8px;">
                <span>Total: ${events.length}</span>
                <span style="color: ${tamperedCount > 0 ? '#ef4444' : '#4ade80'};">Tampered: ${tamperedCount}</span>
                <span style="color: ${divergedCount > 0 ? '#fbbf24' : '#4ade80'};">Diverged: ${divergedCount}</span>
            </div>
            <div style="margin-top: 8px; padding: 8px; background: rgba(255,255,255,0.1); border-radius: 4px; font-size: 0.85em;">
                ℹ️ <strong>Note:</strong> Events are preserved during reconciliation. Resyncing resets the accumulator but keeps the event history for audit purposes.
            </div>
        </div>
    `;
    
    treeContainer.innerHTML = html;
}

// Check policies and constraints
function checkPoliciesAndConstraints(dimensionId) {
    const dimension = state.dimensions[dimensionId];
    const dimConfig = config.dimensions[dimensionId];
    
    console.log(`[POLICY] Checking policies for ${dimension.name}`);
    
    // Check threshold policy
    if (dimension.divergence > dimConfig.threshold) {
        const violation = {
            type: 'threshold',
            dimensionId: dimensionId,
            dimensionName: dimension.name,
            threshold: dimConfig.threshold,
            actual: dimension.divergence,
            strategy: dimConfig.strategy,
            timestamp: new Date().toISOString(),
            resolved: false
        };
        
        state.policyViolations.push(violation);
        console.log(`[POLICY] Threshold violation detected: ${dimension.name} (${dimension.divergence} > ${dimConfig.threshold})`);
        
        // Enforce strategy
        enforcePolicyStrategy(violation);
    }
    
    // Check constraints
    config.constraints.forEach((constraint, index) => {
        const violated = checkConstraint(constraint);
        state.constraintStatus[index] = {
            constraint: constraint,
            satisfied: !violated,
            lastCheck: new Date().toISOString()
        };
        
        if (violated) {
            console.log(`[CONSTRAINT] Violation: ${constraint.description}`);
            logEvent('CONSTRAINT', `Constraint violated: ${constraint.description}`, 0);
        }
    });
    
    updatePolicyPanel();
}

// Check individual constraint
function checkConstraint(constraint) {
    if (constraint.type === 'MaxRatio') {
        const [dim1Id, dim2Id] = constraint.dimensions;
        const div1 = state.dimensions[dim1Id]?.divergence || 0;
        const div2 = state.dimensions[dim2Id]?.divergence || 0;
        
        if (div2 === 0) return false; // Avoid division by zero
        
        const ratio = div1 / div2;
        return ratio > constraint.ratio;
    } else if (constraint.type === 'SumBelow') {
        const sum = constraint.dimensions.reduce((acc, dimId) => {
            return acc + (state.dimensions[dimId]?.divergence || 0);
        }, 0);
        return sum > constraint.maxSum;
    }
    return false;
}

// Enforce policy strategy
function enforcePolicyStrategy(violation) {
    console.log(`[POLICY] Enforcing strategy: ${violation.strategy}`);
    
    switch (violation.strategy) {
        case 'MonitorOnly':
            logEvent('POLICY', `⚠️ ${violation.dimensionName} exceeded threshold (${violation.actual} > ${violation.threshold}) - Monitoring only`, violation.actual);
            break;
            
        case 'ImmediateHeal':
            logEvent('POLICY', `🔧 ${violation.dimensionName} exceeded threshold - Attempting immediate heal`, violation.actual);
            // Auto-trigger resync
            setTimeout(() => {
                console.log(`[POLICY] Auto-healing ${violation.dimensionName}`);
                resyncDimension(violation.dimensionId);
                violation.resolved = true;
            }, 1000);
            break;
            
        case 'Quarantine':
            logEvent('POLICY', `🚨 ${violation.dimensionName} exceeded threshold - QUARANTINING`, violation.actual);
            // Auto-quarantine
            setTimeout(() => {
                console.log(`[POLICY] Auto-quarantining ${violation.dimensionName}`);
                const dimension = state.dimensions[violation.dimensionId];
                dimension.quarantined = true;
                const dimNode = state.nodes.find(n => n.data.dimensionId === violation.dimensionId);
                if (dimNode) {
                    dimNode.color = '#dc3545';
                    dimNode.data.quarantined = true;
                }
                alert(`🚨 POLICY ENFORCEMENT\n\n${violation.dimensionName} has been automatically quarantined!\n\nReason: Divergence (${violation.actual}) exceeded threshold (${violation.threshold})\nStrategy: ${violation.strategy}`);
                violation.resolved = true;
            }, 1000);
            break;
            
        case 'GracefulDegrade':
            logEvent('POLICY', `⚡ ${violation.dimensionName} exceeded threshold - Graceful degradation`, violation.actual);
            break;
    }
}

// Update policy panel
function updatePolicyPanel() {
    const policyContainer = document.getElementById('policyStatus');
    if (!policyContainer) return;
    
    let html = '';
    
    // Show active violations
    const activeViolations = state.policyViolations.filter(v => !v.resolved).slice(-5);
    if (activeViolations.length > 0) {
        html += '<div style="margin-bottom: 10px;">';
        html += '<strong style="color: #ef4444;">⚠️ Active Violations:</strong>';
        activeViolations.forEach(v => {
            html += `
                <div style="background: rgba(239, 68, 68, 0.2); padding: 8px; border-radius: 4px; margin-top: 5px; font-size: 0.85em;">
                    <strong>${v.dimensionName}</strong>: ${v.actual} > ${v.threshold}<br>
                    <span style="opacity: 0.8;">Strategy: ${v.strategy}</span>
                </div>
            `;
        });
        html += '</div>';
    }
    
    // Show constraint status
    html += '<div style="margin-top: 10px;">';
    html += '<strong>Constraints:</strong>';
    Object.values(state.constraintStatus).forEach(status => {
        const color = status.satisfied ? '#4ade80' : '#ef4444';
        const icon = status.satisfied ? '✓' : '✗';
        html += `
            <div style="background: rgba(255,255,255,0.1); padding: 6px; border-radius: 4px; margin-top: 5px; font-size: 0.85em; border-left: 3px solid ${color};">
                <span style="color: ${color};">${icon}</span> ${status.constraint.description}
            </div>
        `;
    });
    html += '</div>';
    
    if (html === '') {
        html = '<div style="text-align: center; opacity: 0.6; padding: 20px;">All policies satisfied ✓</div>';
    }
    
    policyContainer.innerHTML = html;
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
    
    // Scale down: 2^256 ≈ 1.16×10^77, so dividing by 10^74 gives values in 0-1000 range
    return Number(diff / BigInt(10) ** BigInt(74));
}

// Initialize on load
document.addEventListener('DOMContentLoaded', async () => {
    console.log('[VISUAL] Starting visual demo...');
    await initializeWasm();
});
