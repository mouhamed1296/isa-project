// MA-ISA Mobile Money Platform - Next-Generation Financial Integrity
import init, { WasmAxisAccumulator } from './pkg/isa_ffi.js';

// State management
const state = {
    wasmInitialized: false,
    tamperMode: false,
    dimensions: [],
    accumulators: [],
    referenceAccumulators: [],
    eventLog: [],
    dimensionEventHistory: {},
    totalEvents: 0,
    canvas: null,
    ctx: null,
    nodes: [],
    animations: [],
    selectedDimensionId: null,
    policyViolations: [],
    constraintStatus: {}
};

// Mobile Money configuration - Future-focused dimensions
const config = {
    dimensions: [
        { id: 0, name: "Cross-Border Payments", color: "#3b82f6", threshold: 180, weight: 3.0, strategy: "ImmediateHeal", safetyRelevant: true },
        { id: 1, name: "AI Fraud Detection", color: "#8b5cf6", threshold: 100, weight: 3.5, strategy: "Quarantine", safetyRelevant: true },
        { id: 2, name: "DeFi Integration", color: "#ec4899", threshold: 220, weight: 2.5, strategy: "ImmediateHeal", safetyRelevant: true },
        { id: 3, name: "Micropayment Network", color: "#f59e0b", threshold: 300, weight: 2.0, strategy: "MonitorOnly", safetyRelevant: false }
    ],
    constraints: [
        {
            type: "MaxRatio",
            dimensions: [0, 2],
            ratio: 1.2,
            description: "Cross-border payment divergence must not exceed 1.2x DeFi integration divergence"
        },
        {
            type: "SumBelow",
            dimensions: [0, 1, 2],
            maxSum: 600,
            description: "Total divergence across critical payment systems must stay below 600"
        },
        {
            type: "MaxRatio",
            dimensions: [1, 3],
            ratio: 0.5,
            description: "AI fraud detection divergence must stay below 0.5x micropayment divergence (fraud detection is critical)"
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
        this.type = type;
        this.label = label;
        this.data = data;
        this.radius = type === 'root' ? 40 : 35;
        this.color = data.color || '#10b981';
        this.divergence = 0;
        this.pulsePhase = 0;
        this.parent = null;
    }

    draw(ctx) {
        if (this.parent) {
            ctx.strokeStyle = this.divergence > 0 ? '#f87171' : 'rgba(255, 255, 255, 0.3)';
            ctx.lineWidth = this.divergence > 0 ? 3 : 2;
            ctx.beginPath();
            ctx.moveTo(this.parent.x, this.parent.y);
            ctx.lineTo(this.x, this.y);
            ctx.stroke();

            if (this.divergence > 0) {
                const midX = (this.parent.x + this.x) / 2;
                const midY = (this.parent.y + this.y) / 2;
                ctx.fillStyle = '#f87171';
                ctx.font = 'bold 12px Arial';
                ctx.fillText(`⚠️ ${this.divergence}`, midX + 10, midY - 10);
            }
        }
        
        const isSelected = this.type === 'dimension' && state.selectedDimensionId === this.data.dimensionId;
        if (isSelected) {
            ctx.strokeStyle = '#fbbf24';
            ctx.lineWidth = 4;
            ctx.beginPath();
            ctx.arc(this.x, this.y, this.radius + 5, 0, Math.PI * 2);
            ctx.stroke();
        }

        if (this.divergence > 0) {
            this.pulsePhase += 0.1;
            const pulseScale = 1 + Math.sin(this.pulsePhase) * 0.2;
            
            ctx.strokeStyle = '#f87171';
            ctx.lineWidth = 3;
            ctx.globalAlpha = 0.5 + Math.sin(this.pulsePhase) * 0.3;
            ctx.beginPath();
            ctx.arc(this.x, this.y, this.radius * pulseScale * 1.5, 0, Math.PI * 2);
            ctx.stroke();
            ctx.globalAlpha = 1;
        }

        ctx.fillStyle = this.color;
        ctx.beginPath();
        ctx.arc(this.x, this.y, this.radius, 0, Math.PI * 2);
        ctx.fill();

        ctx.strokeStyle = 'rgba(255, 255, 255, 0.5)';
        ctx.lineWidth = 2;
        ctx.stroke();

        ctx.fillStyle = '#fff';
        ctx.font = 'bold 14px Arial';
        ctx.textAlign = 'center';
        ctx.textBaseline = 'middle';
        
        // Shorten labels for display
        let displayLabel = this.label;
        if (this.type === 'dimension') {
            const shortLabels = {
                'Cross-Border': 'X-Border',
                'AI Fraud': 'AI Fraud',
                'DeFi': 'DeFi',
                'Micropayment': 'Micro'
            };
            displayLabel = shortLabels[this.label] || this.label;
        }
        
        ctx.fillText(displayLabel, this.x, this.y);

        if (this.type === 'dimension' && this.data.eventCount > 0) {
            ctx.fillStyle = '#000';
            ctx.fillRect(this.x + this.radius - 15, this.y - this.radius, 20, 16);
            ctx.fillStyle = '#fff';
            ctx.font = 'bold 10px Arial';
            ctx.fillText(this.data.eventCount, this.x + this.radius - 5, this.y - this.radius + 8);
        }
    }
}

class AccumulationAnimation {
    constructor(from, to, eventData) {
        this.from = from;
        this.to = to;
        this.eventData = eventData;
        this.progress = 0;
        this.speed = 0.02;
        this.particles = [];
        
        for (let i = 0; i < 3; i++) {
            this.particles.push({
                offset: i * 0.3,
                size: 4 + Math.random() * 3
            });
        }
    }

    update() {
        this.progress += this.speed;
        return this.progress >= 1;
    }

    draw(ctx) {
        this.particles.forEach(particle => {
            const p = Math.min(1, this.progress + particle.offset);
            if (p > 1) return;

            const x = this.from.x + (this.to.x - this.from.x) * p;
            const y = this.from.y + (this.to.y - this.from.y) * p;

            ctx.fillStyle = this.to.color;
            ctx.globalAlpha = 1 - p;
            ctx.beginPath();
            ctx.arc(x, y, particle.size, 0, Math.PI * 2);
            ctx.fill();
            ctx.globalAlpha = 1;
        });
    }
}

async function initializeWasm() {
    try {
        console.log('[WASM] Initializing...');
        await init();
        console.log('[WASM] Initialized successfully');

        state.dimensions = config.dimensions.map(dim => ({
            id: dim.id,
            name: dim.name,
            state: '',
            eventCount: 0,
            divergence: 0,
            quarantined: false
        }));

        const seed = new Uint8Array(32);
        crypto.getRandomValues(seed);

        state.accumulators = [];
        state.referenceAccumulators = [];

        for (let i = 0; i < config.dimensions.length; i++) {
            const accumulator = new WasmAxisAccumulator(seed);
            const refAccumulator = new WasmAxisAccumulator(seed);
            
            state.accumulators.push(accumulator);
            state.referenceAccumulators.push(refAccumulator);
            state.dimensions[i].state = bytesToHex(accumulator.state());
            
            console.log(`[WASM] Dimension ${i} (${config.dimensions[i].name}) initialized`);
        }

        state.wasmInitialized = true;
        updateStatus();
        initializeMindMap();
        logEvent('SYSTEM', 'Mobile money platform initialized - All dimensions operational', 0);
        
    } catch (error) {
        console.error('[WASM] Initialization failed:', error);
        logEvent('SYSTEM', `Initialization failed: ${error.message}`, 0);
    }
}

function initializeMindMap() {
    const canvas = document.getElementById('mindMapCanvas');
    const ctx = canvas.getContext('2d');
    
    canvas.width = canvas.offsetWidth;
    canvas.height = canvas.offsetHeight;
    
    state.canvas = canvas;
    state.ctx = ctx;
    
    canvas.addEventListener('click', handleCanvasClick);
    
    const rootNode = new Node(
        canvas.width / 2,
        canvas.height / 2,
        'root',
        '💰',
        { color: '#10b981' }
    );
    state.nodes = [rootNode];
    
    const angleStep = (Math.PI * 2) / config.dimensions.length;
    const radius = 200;
    
    config.dimensions.forEach((dim, index) => {
        const angle = index * angleStep - Math.PI / 2;
        const x = rootNode.x + Math.cos(angle) * radius;
        const y = rootNode.y + Math.sin(angle) * radius;
        
        const shortLabels = {
            'Cross-Border Payments': 'Cross-Border',
            'AI Fraud Detection': 'AI Fraud',
            'DeFi Integration': 'DeFi',
            'Micropayment Network': 'Micropayment'
        };
        
        const dimNode = new Node(x, y, 'dimension', shortLabels[dim.name] || dim.name, {
            dimensionId: dim.id,
            color: dim.color,
            eventCount: 0,
            quarantined: false
        });
        dimNode.parent = rootNode;
        state.nodes.push(dimNode);
    });
    
    animate();
}

function animate() {
    const ctx = state.ctx;
    const canvas = state.canvas;
    
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    
    state.nodes.forEach(node => node.draw(ctx));
    
    state.animations = state.animations.filter(anim => {
        anim.draw(ctx);
        return !anim.update();
    });
    
    requestAnimationFrame(animate);
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

window.recordEvent = function(dimensionId) {
    if (!state.wasmInitialized) {
        alert('System not initialized yet!');
        return;
    }

    const dimension = state.dimensions[dimensionId];
    if (dimension.quarantined) {
        alert(`⚠️ ${dimension.name} is quarantined and cannot accept events!`);
        return;
    }

    const accumulator = state.accumulators[dimensionId];
    const refAccumulator = state.referenceAccumulators[dimensionId];
    
    const eventData = generateEventData(dimensionId);
    const eventBytes = new TextEncoder().encode(eventData);
    
    const entropy = new Uint8Array(32);
    crypto.getRandomValues(entropy);
    
    const deltaT = BigInt(Date.now());
    
    console.log(`[EVENT] Recording for dimension ${dimensionId}: ${eventData}`);
    
    let activeEventBytes = eventBytes;
    if (state.tamperMode) {
        const tamperedData = eventData + '_TAMPERED';
        activeEventBytes = new TextEncoder().encode(tamperedData);
        console.log(`[TAMPER] Modified event data: ${tamperedData}`);
    }
    
    accumulator.accumulate(activeEventBytes, entropy, deltaT);
    refAccumulator.accumulate(eventBytes, entropy, deltaT);
    
    const newState = accumulator.state();
    const refState = refAccumulator.state();
    dimension.state = bytesToHex(newState);
    dimension.eventCount++;
    dimension.divergence = calculateDivergence(newState, refState);
    
    state.totalEvents++;
    
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
    
    const dimNode = state.nodes.find(n => n.data.dimensionId === dimensionId);
    if (dimNode) {
        dimNode.data.eventCount = dimension.eventCount;
        dimNode.divergence = dimension.divergence;
        
        const rootNode = state.nodes[0];
        const animation = new AccumulationAnimation(rootNode, dimNode, eventData);
        state.animations.push(animation);
    }
    
    logEvent(dimension.name, eventData, dimension.divergence);
    
    checkPoliciesAndConstraints(dimensionId);
    
    updateStatus();
    
    if (state.selectedDimensionId === dimensionId) {
        updateEventTree(dimensionId);
    }
    
    console.log('[EVENT] Recorded:', { dimensionId, eventData, divergence: dimension.divergence });
};

function generateEventData(dimensionId) {
    switch (dimensionId) {
        case 0: // Cross-Border Payments
            const corridors = [
                'KE_to_UG', 'NG_to_GH', 'ZA_to_ZW', 'US_to_MX', 'FR_to_SN',
                'UK_to_IN', 'CN_to_PH', 'SA_to_PK', 'AE_to_BD', 'SG_to_MY'
            ];
            const methods = ['instant_settlement', 'blockchain_bridge', 'stablecoin_transfer', 'cbdc_exchange'];
            const amount = (Math.random() * 5000 + 100).toFixed(2);
            return `xborder_${methods[Math.floor(Math.random() * methods.length)]}_${corridors[Math.floor(Math.random() * corridors.length)]}_$${amount}`;
        
        case 1: // AI Fraud Detection
            const fraudTypes = ['velocity_check', 'pattern_anomaly', 'geolocation_mismatch', 'device_fingerprint', 'behavioral_analysis', 'ml_risk_score'];
            const riskLevels = ['low', 'medium', 'high', 'critical'];
            const actions = ['approved', 'flagged', 'blocked', 'reviewed'];
            const score = (Math.random() * 100).toFixed(1);
            return `fraud_${fraudTypes[Math.floor(Math.random() * fraudTypes.length)]}_risk_${riskLevels[Math.floor(Math.random() * riskLevels.length)]}_${actions[Math.floor(Math.random() * actions.length)]}_score_${score}`;
        
        case 2: // DeFi Integration
            const defiProtocols = ['uniswap', 'aave', 'compound', 'curve', 'yearn', 'balancer', 'sushiswap'];
            const operations = ['liquidity_provision', 'yield_farming', 'token_swap', 'lending', 'borrowing', 'staking'];
            const tokens = ['USDC', 'DAI', 'USDT', 'ETH', 'BTC', 'MATIC'];
            const value = (Math.random() * 10000 + 500).toFixed(2);
            return `defi_${operations[Math.floor(Math.random() * operations.length)]}_${defiProtocols[Math.floor(Math.random() * defiProtocols.length)]}_${tokens[Math.floor(Math.random() * tokens.length)]}_$${value}`;
        
        case 3: // Micropayment Network
            const microTypes = ['content_tip', 'api_call', 'data_packet', 'streaming_second', 'game_item', 'article_read', 'ad_view'];
            const platforms = ['social_media', 'gaming', 'streaming', 'iot_device', 'api_service', 'content_platform'];
            const microAmount = (Math.random() * 0.5 + 0.01).toFixed(4);
            return `micro_${microTypes[Math.floor(Math.random() * microTypes.length)]}_${platforms[Math.floor(Math.random() * platforms.length)]}_$${microAmount}`;
        
        default:
            return `event_${Date.now()}`;
    }
}

window.toggleTamper = function() {
    state.tamperMode = !state.tamperMode;
    const btn = document.getElementById('tamperBtn');
    const status = document.getElementById('tamperStatus').querySelector('.status-value');
    
    if (state.tamperMode) {
        btn.textContent = '🔒 Disable Tamper Mode';
        btn.className = 'btn-success';
        status.textContent = 'TAMPER MODE ACTIVE';
        status.className = 'status-value status-error';
        logEvent('SYSTEM', '⚠️ Tamper mode enabled - Events will be modified', 0);
    } else {
        btn.textContent = '🔓 Enable Tamper Mode';
        btn.className = 'btn-danger';
        status.textContent = 'Normal';
        status.className = 'status-value status-ok';
        logEvent('SYSTEM', 'Tamper mode disabled', 0);
    }
};

window.openReconciliation = function() {
    const modal = document.getElementById('reconciliationModal');
    const content = document.getElementById('reconciliationContent');
    
    let html = '<div style="display: flex; flex-direction: column; gap: 20px;">';
    
    state.dimensions.forEach((dimension, index) => {
        const dimConfig = config.dimensions[index];
        const activeState = dimension.state.substring(0, 32) + '...';
        const refState = bytesToHex(state.referenceAccumulators[index].state()).substring(0, 32) + '...';
        const isDiverged = dimension.divergence > 0;
        
        html += `
            <div style="background: rgba(255,255,255,0.1); padding: 20px; border-radius: 10px; border-left: 5px solid ${isDiverged ? '#ef4444' : '#4ade80'};">
                <h3 style="margin-bottom: 15px; display: flex; align-items: center; gap: 10px;">
                    <span style="color: ${dimConfig.color};">●</span>
                    ${dimension.name}
                    ${isDiverged ? '<span style="color: #ef4444; font-size: 0.9em;">⚠️ DIVERGED</span>' : '<span style="color: #4ade80; font-size: 0.9em;">✓ SYNCED</span>'}
                </h3>
                
                <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 15px; margin-bottom: 15px;">
                    <div>
                        <strong>Active State:</strong><br>
                        <code style="font-size: 0.85em; word-break: break-all;">${activeState}</code>
                    </div>
                    <div>
                        <strong>Reference State:</strong><br>
                        <code style="font-size: 0.85em; word-break: break-all;">${refState}</code>
                    </div>
                </div>
                
                <div style="display: flex; gap: 10px; align-items: center;">
                    <strong>Divergence:</strong> <span style="color: ${isDiverged ? '#ef4444' : '#4ade80'};">${dimension.divergence}</span>
                    <strong style="margin-left: 20px;">Events:</strong> ${dimension.eventCount}
                    <strong style="margin-left: 20px;">Strategy:</strong> ${dimConfig.strategy}
                </div>
                
                ${isDiverged ? `
                    <div style="margin-top: 15px; display: flex; gap: 10px;">
                        <button onclick="resyncDimension(${index})" style="padding: 8px 16px; background: #4ade80; color: #000; border: none; border-radius: 5px; cursor: pointer; font-weight: bold;">🔄 Resync to Reference</button>
                        <button onclick="quarantineDimension(${index})" style="padding: 8px 16px; background: #ef4444; color: white; border: none; border-radius: 5px; cursor: pointer; font-weight: bold;">🚨 Quarantine Dimension</button>
                        <button onclick="triggerAlert(${index})" style="padding: 8px 16px; background: #fbbf24; color: #000; border: none; border-radius: 5px; cursor: pointer; font-weight: bold;">⚠️ Trigger Alert</button>
                    </div>
                ` : ''}
            </div>
        `;
    });
    
    html += `
        <div style="background: rgba(255,255,255,0.05); padding: 15px; border-radius: 10px; font-size: 0.9em;">
            <strong>📖 Reconciliation Guide:</strong>
            <ul style="margin-top: 10px; padding-left: 20px;">
                <li><strong>Resync:</strong> Reset active state to match reference (clears divergence)</li>
                <li><strong>Quarantine:</strong> Isolate dimension and prevent further events</li>
                <li><strong>Alert:</strong> Generate security alert for manual review</li>
            </ul>
        </div>
    `;
    
    html += '</div>';
    content.innerHTML = html;
    modal.style.display = 'flex';
};

window.closeReconciliation = function() {
    document.getElementById('reconciliationModal').style.display = 'none';
};

window.resyncDimension = function(dimensionId) {
    const dimension = state.dimensions[dimensionId];
    
    if (!confirm(`Resync ${dimension.name} to reference state? This will clear the divergence.`)) {
        return;
    }
    
    const seed = new Uint8Array(32);
    crypto.getRandomValues(seed);
    const newAccumulator = new WasmAxisAccumulator(seed);
    const newRefAccumulator = new WasmAxisAccumulator(seed);
    
    state.accumulators[dimensionId] = newAccumulator;
    state.referenceAccumulators[dimensionId] = newRefAccumulator;
    
    dimension.state = bytesToHex(newAccumulator.state());
    dimension.divergence = 0;
    dimension.eventCount = 0;
    
    const dimNode = state.nodes.find(n => n.data.dimensionId === dimensionId);
    if (dimNode) {
        dimNode.divergence = 0;
        dimNode.data.eventCount = 0;
    }
    
    logEvent('RECONCILIATION', `${dimension.name} resynced to reference state (${state.dimensionEventHistory[dimensionId]?.length || 0} events preserved)`, 0);
    updateStatus();
    openReconciliation();
    
    if (state.selectedDimensionId === dimensionId) {
        updateEventTree(dimensionId);
    }
    
    console.log(`[RESYNC] Dimension ${dimensionId} resynced successfully`);
    console.log(`[RESYNC] Event history preserved: ${state.dimensionEventHistory[dimensionId]?.length || 0} events`);
};

window.quarantineDimension = function(dimensionId) {
    const dimension = state.dimensions[dimensionId];
    
    if (!confirm(`Quarantine ${dimension.name}? This will isolate the dimension and prevent further events until resolved.`)) {
        return;
    }
    
    dimension.quarantined = true;
    
    const dimNode = state.nodes.find(n => n.data.dimensionId === dimensionId);
    if (dimNode) {
        dimNode.color = '#dc3545';
        dimNode.data.quarantined = true;
    }
    
    logEvent('SECURITY', `🚨 ${dimension.name} QUARANTINED - Divergence: ${dimension.divergence}`, dimension.divergence);
    
    alert(`🚨 QUARANTINE ACTIVATED\n\n${dimension.name} has been quarantined!\n\nDivergence: ${dimension.divergence}\nEvents: ${dimension.eventCount}\n\nNo further events will be accepted until the dimension is restored.`);
    
    closeReconciliation();
};

window.triggerAlert = function(dimensionId) {
    const dimension = state.dimensions[dimensionId];
    const dimConfig = config.dimensions[dimensionId];
    
    const alertMessage = `
🚨 SECURITY ALERT - MOBILE MONEY PLATFORM

Dimension: ${dimension.name}
Divergence: ${dimension.divergence}
Threshold: ${dimConfig.threshold}
Events Recorded: ${dimension.eventCount}
Strategy: ${dimConfig.strategy}

State Preview: ${dimension.state.substring(0, 32)}...

Recommended Actions:
1. Review event history for tampering
2. Investigate source of divergence
3. Consider resyncing or quarantining
4. Notify security team
    `;
    
    console.warn('[ALERT]', alertMessage);
    logEvent('ALERT', `Security alert triggered for ${dimension.name} (Δ${dimension.divergence})`, dimension.divergence);
    
    alert(alertMessage);
};

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

function updateStatus() {
    document.getElementById('wasmStatus').textContent = state.wasmInitialized ? '✅ Ready' : '⏳ Loading';
    document.getElementById('wasmStatus').className = state.wasmInitialized ? 'status-value status-ok' : 'status-value status-warning';
    
    document.getElementById('dimensionCount').textContent = state.dimensions.length;
    document.getElementById('eventCount').textContent = state.totalEvents;
    
    const maxDivergence = Math.max(...state.dimensions.map(d => d.divergence));
    const overallStatus = document.getElementById('overallStatus');
    
    if (maxDivergence === 0) {
        overallStatus.textContent = '✅ Secure';
        overallStatus.className = 'status-value status-ok';
    } else if (maxDivergence < 300) {
        overallStatus.textContent = '⚠️ Warning';
        overallStatus.className = 'status-value status-warning';
    } else {
        overallStatus.textContent = '🚨 Critical';
        overallStatus.className = 'status-value status-error';
    }
}

function logEvent(dimension, data, divergence) {
    const timestamp = new Date().toLocaleTimeString();
    state.eventLog.unshift({ timestamp, dimension, data, divergence });
    
    if (state.eventLog.length > 20) {
        state.eventLog.pop();
    }
    
    const logContainer = document.getElementById('eventLog');
    logContainer.innerHTML = state.eventLog.map(e => `
        <div class="event-entry">
            <span class="event-timestamp">${e.timestamp}</span> 
            <strong>${e.dimension}:</strong> ${e.data}
            ${e.divergence > 0 ? `<span class="status-error"> ⚠️ Δ${e.divergence}</span>` : ''}
        </div>
    `).join('');
}

function handleCanvasClick(event) {
    const rect = state.canvas.getBoundingClientRect();
    const x = event.clientX - rect.left;
    const y = event.clientY - rect.top;
    
    for (const node of state.nodes) {
        if (node.type === 'dimension') {
            const distance = Math.sqrt((x - node.x) ** 2 + (y - node.y) ** 2);
            if (distance <= node.radius) {
                state.selectedDimensionId = node.data.dimensionId;
                updateEventTree(node.data.dimensionId);
                console.log(`[SELECT] Dimension ${node.data.dimensionId} selected`);
                return;
            }
        }
    }
    
    if (state.selectedDimensionId !== null) {
        state.selectedDimensionId = null;
        updateEventTree(null);
        console.log('[SELECT] Dimension deselected');
    }
}

function updateEventTree(dimensionId) {
    const treeContainer = document.getElementById('eventTree');
    const titleEl = document.getElementById('eventTreeTitle');
    const subtitleEl = document.getElementById('eventTreeSubtitle');
    
    if (dimensionId === null) {
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

function checkPoliciesAndConstraints(dimensionId) {
    const dimension = state.dimensions[dimensionId];
    const dimConfig = config.dimensions[dimensionId];
    
    console.log(`[POLICY] Checking policies for ${dimension.name}`);
    
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
        
        enforcePolicyStrategy(violation);
    }
    
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

function checkConstraint(constraint) {
    if (constraint.type === 'MaxRatio') {
        const [dim1Id, dim2Id] = constraint.dimensions;
        const div1 = state.dimensions[dim1Id]?.divergence || 0;
        const div2 = state.dimensions[dim2Id]?.divergence || 0;
        
        if (div2 === 0) return false;
        
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

function enforcePolicyStrategy(violation) {
    console.log(`[POLICY] Enforcing strategy: ${violation.strategy}`);
    
    switch (violation.strategy) {
        case 'MonitorOnly':
            logEvent('POLICY', `⚠️ ${violation.dimensionName} exceeded threshold (${violation.actual} > ${violation.threshold}) - Monitoring only`, violation.actual);
            break;
            
        case 'ImmediateHeal':
            logEvent('POLICY', `🔧 ${violation.dimensionName} exceeded threshold - Attempting immediate heal`, violation.actual);
            setTimeout(() => {
                console.log(`[POLICY] Auto-healing ${violation.dimensionName}`);
                resyncDimension(violation.dimensionId);
                violation.resolved = true;
            }, 1000);
            break;
            
        case 'Quarantine':
            logEvent('POLICY', `🚨 ${violation.dimensionName} exceeded threshold - QUARANTINING`, violation.actual);
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

function updatePolicyPanel() {
    const policyContainer = document.getElementById('policyStatus');
    if (!policyContainer) return;
    
    let html = '';
    
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

function bytesToHex(bytes) {
    return Array.from(bytes).map(b => b.toString(16).padStart(2, '0')).join('');
}

document.addEventListener('DOMContentLoaded', () => {
    console.log('[INIT] Starting mobile money platform demo...');
    initializeWasm();
});
