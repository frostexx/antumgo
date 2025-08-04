// Quantum Bot Enhancement - Ultimate Supremacy Controller
class QuantumBotController {
    constructor() {
        this.isLoggedIn = false;
        this.quantumMode = true;
        this.webSocket = null;
        this.metrics = {
            networkDominance: 0,
            operationsPerSec: 0,
            competitorsDefeated: 0,
            claimAttempts: 0,
            transferAttempts: 0
        };
        this.competitorActivity = {};
        this.logEntries = [];
        this.serverTimeOffset = 0;
        
        this.init();
    }

    init() {
        this.setupEventListeners();
        this.startServerTimeSync();
        this.startQuantumAnimations();
        this.initializeQuantumEffects();
        
        console.log('🚀 Quantum Bot Enhancement Initialized');
        console.log('⚡ Ultimate Supremacy Mode: ACTIVE');
    }

    setupEventListeners() {
        // Navigation
        document.getElementById('loginTab')?.addEventListener('click', () => this.showSection('login'));
        document.getElementById('withdrawTab')?.addEventListener('click', () => this.showSection('withdraw'));

        // Forms
        document.getElementById('loginForm')?.addEventListener('submit', (e) => this.handleLogin(e));
        document.getElementById('withdrawForm')?.addEventListener('submit', (e) => this.handleWithdraw(e));

        // Quantum mode toggle
        document.getElementById('quantumMode')?.addEventListener('change', (e) => {
            this.quantumMode = e.target.checked;
            this.updateQuantumUI();
        });

        // Real-time input validation
        document.getElementById('seedPhrase')?.addEventListener('input', this.validateSeedPhrase.bind(this));
        document.getElementById('sponsorPhrase')?.addEventListener('input', this.validateSponsorPhrase.bind(this));
        document.getElementById('withdrawAddress')?.addEventListener('input', this.validateAddress.bind(this));
    }

    showSection(section) {
        // Hide all sections
        document.querySelectorAll('.quantum-section').forEach(sec => {
            sec.classList.remove('active');
        });

        // Remove active from nav buttons
        document.querySelectorAll('.nav-btn').forEach(btn => {
            btn.classList.remove('active');
        });

        // Show selected section
        const sectionElement = section === 'login' ? 
            document.getElementById('loginSection') : 
            document.getElementById('withdrawSection');
        
        const tabElement = section === 'login' ? 
            document.getElementById('loginTab') : 
            document.getElementById('withdrawTab');

        sectionElement?.classList.add('active');
        tabElement?.classList.add('active');

        // Trigger quantum animation
        this.triggerQuantumTransition();
    }

    async handleLogin(event) {
        event.preventDefault();
        
        const seedPhrase = document.getElementById('seedPhrase').value.trim();
        
        if (!this.validateSeedPhrase(seedPhrase)) {
            this.showError('Invalid seed phrase format');
            return;
        }

        this.showLoading('Authenticating with Quantum Systems...');

        try {
            const response = await fetch('/api/login', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    seed_phrase: seedPhrase
                })
            });

            const data = await response.json();

            if (response.ok) {
                this.isLoggedIn = true;
                this.displayAccountInfo(data);
                this.populateLockedBalances(data.locked_balances);
                this.displayRecentTransactions(data.transactions);
                this.showSuccess('🚀 Quantum Authentication Successful!');
                this.startQuantumMetrics();
            } else {
                this.showError(data.message || 'Authentication failed');
            }
        } catch (error) {
            this.showError('Network error: ' + error.message);
        } finally {
            this.hideLoading();
        }
    }

    async handleWithdraw(event) {
        event.preventDefault();

        if (!this.isLoggedIn) {
            this.showError('Please login first');
            return;
        }

        const formData = this.getWithdrawFormData();
        
        if (!this.validateWithdrawForm(formData)) {
            return;
        }

        this.showSuccess('🚀 Initiating Quantum Supremacy Protocol...');
        this.startQuantumWithdrawal(formData);
    }

    getWithdrawFormData() {
        return {
            seed_phrase: document.getElementById('seedPhrase').value.trim(),
            sponsor_phrase: document.getElementById('sponsorPhrase').value.trim(),
            locked_balance_id: document.getElementById('lockedBalance').value,
            withdrawal_address: document.getElementById('withdrawAddress').value.trim(),
            amount: document.getElementById('withdrawAmount').value,
            unlock_time: document.getElementById('unlockTime').value,
            quantum_mode: document.getElementById('quantumMode').checked
        };
    }

    validateWithdrawForm(data) {
        if (!data.withdrawal_address) {
            this.showError('Withdrawal address is required');
            return false;
        }

        if (!data.locked_balance_id) {
            this.showError('Please select a locked balance');
            return false;
        }

        if (!data.amount || parseFloat(data.amount) <= 0) {
            this.showError('Invalid withdrawal amount');
            return false;
        }

        if (!data.unlock_time) {
            this.showError('Unlock time is required');
            return false;
        }

        return true;
    }

    startQuantumWithdrawal(formData) {
        // Close existing WebSocket if any
        if (this.webSocket) {
            this.webSocket.close();
        }

        // Create WebSocket connection
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const wsUrl = `${protocol}//${window.location.host}/ws/withdraw`;
        
        this.webSocket = new WebSocket(wsUrl);

        this.webSocket.onopen = () => {
            this.addLog('🌐 Quantum communication channel established', 'quantum');
            this.webSocket.send(JSON.stringify(formData));
        };

        this.webSocket.onmessage = (event) => {
            try {
                const response = JSON.parse(event.data);
                this.handleQuantumResponse(response);
            } catch (error) {
                this.addLog('❌ Error parsing response: ' + error.message, 'error');
            }
        };

        this.webSocket.onerror = (error) => {
            this.addLog('❌ Quantum communication error: ' + error.message, 'error');
        };

        this.webSocket.onclose = () => {
            this.addLog('🌐 Quantum communication channel closed', 'info');
        };
    }

    handleQuantumResponse(response) {
        // Update server time
        if (response.server_time) {
            this.updateServerTime(response.server_time);
        }

        // Update metrics
        if (response.quantum_metrics) {
            this.updateQuantumMetrics(response.quantum_metrics);
        }

        if (response.network_dominance !== undefined) {
            this.updateNetworkDominance(response.network_dominance);
        }

        if (response.competitor_activity) {
            this.updateCompetitorActivity(response.competitor_activity);
        }

        if (response.claim_attempts !== undefined) {
            this.metrics.claimAttempts = response.claim_attempts;
        }

        if (response.transfer_attempts !== undefined) {
            this.metrics.transferAttempts = response.transfer_attempts;
        }

        // Handle different actions
        switch (response.action) {
            case 'initialized':
                this.addLog('🚀 ' + response.message, 'quantum');
                break;
            case 'quantum_prep':
                this.addLog('🧠 ' + response.message, 'quantum');
                this.startQuantumPreparation();
                break;
            case 'quantum_ready':
                this.addLog('⚡ ' + response.message, 'quantum');
                this.startCountdown();
                break;
            case 'quantum_monitor':
                this.addLog(`🔥 ${response.message}`, 'quantum');
                this.updateOperationCounters(response);
                break;
            case 'quantum_success':
                this.addLog('🎯 ' + response.message, 'success');
                this.triggerSuccessCelebration();
                break;
            case 'quantum_complete':
                this.addLog('🏆 ' + response.message, 'success');
                this.triggerVictoryCelebration();
                break;
            case 'quantum_error':
                this.addLog('❌ ' + response.message, 'error');
                break;
            case 'schedule':
                this.addLog('📅 ' + response.message, 'info');
                break;
            case 'waiting':
                this.addLog('⏳ ' + response.message, 'info');
                break;
            case 'claiming':
                this.addLog('💎 ' + response.message, 'info');
                break;
            case 'claimed':
                this.addLog('✅ ' + response.message, 'success');
                break;
            case 'withdrawn':
                this.addLog('💰 ' + response.message, 'success');
                break;
            case 'completed':
                this.addLog('🎉 ' + response.message, 'success');
                this.triggerSuccessCelebration();
                break;
            default:
                if (response.message) {
                    const logType = response.success ? 'success' : 'error';
                    this.addLog(response.message, logType);
                }
        }

        // Update UI
        this.updateQuantumDisplay();
    }

    startQuantumPreparation() {
        this.addLog('🔧 Initializing quantum processors...', 'quantum');
        this.addLog('⚡ Calibrating nanosecond precision timing...', 'quantum');
        this.addLog('🌊 Preparing network domination protocols...', 'quantum');
        this.addLog('💰 Loading economic warfare systems...', 'quantum');
        this.addLog('🧠 Activating AI learning algorithms...', 'quantum');
    }

    startCountdown() {
        this.addLog('🚀 Quantum execution countdown initiated...', 'quantum');
        
        let count = 5;
        const countdown = setInterval(() => {
            if (count > 0) {
                this.addLog(`⚡ T-minus ${count}...`, 'quantum');
                count--;
            } else {
                clearInterval(countdown);
                this.addLog('🔥 QUANTUM SUPREMACY UNLEASHED!', 'quantum');
            }
        }, 1000);
    }

    updateOperationCounters(response) {
        if (response.claim_attempts !== undefined) {
            document.getElementById('claimCounter').textContent = response.claim_attempts;
        }
        if (response.transfer_attempts !== undefined) {
            document.getElementById('transferCounter').textContent = response.transfer_attempts;
        }
    }

    updateQuantumMetrics(metrics) {
        if (metrics.operations_per_sec !== undefined) {
            this.metrics.operationsPerSec = metrics.operations_per_sec;
            document.getElementById('operationsPerSec').textContent = this.formatNumber(metrics.operations_per_sec);
        }

        if (metrics.total_operations !== undefined) {
            this.updateOperationCounter(metrics.total_operations);
        }
    }

    updateNetworkDominance(dominance) {
        this.metrics.networkDominance = dominance;
        const element = document.getElementById('networkDominance');
        if (element) {
            element.style.width = dominance + '%';
            element.textContent = dominance.toFixed(1) + '%';
            
            // Change color based on dominance level
            if (dominance >= 90) {
                element.style.background = 'linear-gradient(90deg, #ff6b35, #ff4757)';
            } else if (dominance >= 70) {
                element.style.background = 'linear-gradient(90deg, #00ff88, #00d4ff)';
            } else {
                element.style.background = 'linear-gradient(90deg, #7b68ee, #00d4ff)';
            }
        }
    }

    updateCompetitorActivity(activity) {
        this.competitorActivity = activity;
        const total = Object.values(activity).reduce((sum, count) => sum + count, 0);
        this.metrics.competitorsDefeated = total;
        document.getElementById('competitorsDefeated').textContent = total;
    }

    displayAccountInfo(data) {
        document.getElementById('walletAddress').textContent = data.wallet_address || 'Unknown';
        document.getElementById('availableBalance').textContent = (data.available_balance || '0') + ' PI';
        document.getElementById('lockedCount').textContent = (data.locked_balances?.length || 0).toString();
        
        document.getElementById('accountInfo').classList.remove('hidden');
    }

    populateLockedBalances(balances) {
        const select = document.getElementById('lockedBalance');
        if (!select) return;

        select.innerHTML = '<option value="">Select locked balance...</option>';
        
        if (balances && balances.length > 0) {
            balances.forEach((balance, index) => {
                const option = document.createElement('option');
                option.value = balance.id;
                option.textContent = `Balance ${index + 1}: ${balance.amount} PI`;
                select.appendChild(option);
            });
        }
    }

    displayRecentTransactions(transactions) {
        const container = document.getElementById('transactionsList');
        if (!container) return;

        if (!transactions || transactions.length === 0) {
            container.innerHTML = '<div class="no-transactions">No recent transactions</div>';
            return;
        }

        container.innerHTML = '';
        transactions.slice(0, 5).forEach(tx => {
            const item = document.createElement('div');
            item.className = 'transaction-item';
            item.innerHTML = `
                <span>${tx.type || 'Transaction'}</span>
                <span>${tx.amount || 'N/A'} PI</span>
            `;
            container.appendChild(item);
        });
    }

    addLog(message, type = 'info') {
        const timestamp = new Date().toLocaleTimeString('en-US', {
            hour12: false,
            hour: '2-digit',
            minute: '2-digit',
            second: '2-digit',
            fractionalSecondDigits: 3
        });

        const logEntry = {
            timestamp,
            message,
            type
        };

        this.logEntries.unshift(logEntry);
        
        // Keep only last 100 entries
        if (this.logEntries.length > 100) {
            this.logEntries = this.logEntries.slice(0, 100);
        }

        this.updateLogDisplay();
    }

    updateLogDisplay() {
        const container = document.getElementById('logContainer');
        if (!container) {
            this.createLogContainer();
            return;
        }

        // Update with latest entries
        container.innerHTML = this.logEntries.slice(0, 20).map(entry => `
            <div class="log-entry ${entry.type}">
                <span class="log-time">[${entry.timestamp}]</span>
                <span class="log-message">${entry.message}</span>
            </div>
        `).join('');

        // Auto-scroll to bottom
        container.scrollTop = container.scrollHeight;
    }

    createLogContainer() {
        // Create live logs section if it doesn't exist
        const withdrawSection = document.getElementById('withdrawSection');
        if (!withdrawSection) return;

        const logsSection = document.createElement('div');
        logsSection.className = 'live-logs';
        logsSection.innerHTML = `
            <h3><i class="fas fa-terminal"></i> Quantum Operations Log</h3>
            <div id="logContainer" class="log-container"></div>
        `;

        withdrawSection.appendChild(logsSection);
    }

    startServerTimeSync() {
        this.updateServerTime();
        
        // Update every second
        setInterval(() => {
            this.updateServerTime();
        }, 1000);
    }

    updateServerTime(serverTime) {
        const now = serverTime ? new Date(serverTime) : new Date();
        const timeString = now.toLocaleString('en-US', {
            year: 'numeric',
            month: '2-digit',
            day: '2-digit',
            hour: '2-digit',
            minute: '2-digit',
            second: '2-digit',
            hour12: false,
            timeZone: 'UTC'
        }) + ' UTC';

        const serverTimeElement = document.getElementById('serverTime');
        if (serverTimeElement) {
            serverTimeElement.textContent = timeString;
        }
    }

    startQuantumMetrics() {
        // Simulate quantum metrics updates
        setInterval(() => {
            this.updateQuantumDisplay();
        }, 500);
    }

    updateQuantumDisplay() {
        // Update operations per second with animation
        this.animateNumber('operationsPerSec', this.metrics.operationsPerSec);
        
        // Update competitors defeated
        this.animateNumber('competitorsDefeated', this.metrics.competitorsDefeated);
    }

    animateNumber(elementId, targetValue) {
        const element = document.getElementById(elementId);
        if (!element) return;

        const currentValue = parseInt(element.textContent) || 0;
        const increment = Math.ceil((targetValue - currentValue) / 10);
        
        if (currentValue !== targetValue) {
            element.textContent = Math.min(currentValue + increment, targetValue);
        }
    }

    formatNumber(num) {
        if (num >= 1000000) {
            return (num / 1000000).toFixed(1) + 'M';
        } else if (num >= 1000) {
            return (num / 1000).toFixed(1) + 'K';
        }
        return num.toString();
    }

    triggerQuantumTransition() {
        // Add quantum transition effect
        document.body.style.filter = 'brightness(1.2) saturate(1.5)';
        
        setTimeout(() => {
            document.body.style.filter = '';
        }, 300);
    }

    triggerSuccessCelebration() {
        this.createParticleEffect(50, '#00ff88');
        this.playSuccessSound();
    }

    triggerVictoryCelebration() {
        this.createParticleEffect(100, '#ff6b35');
        this.createParticleEffect(100, '#00d4ff');
        this.createParticleEffect(100, '#7b68ee');
        this.playVictorySound();
    }

    createParticleEffect(count, color) {
        for (let i = 0; i < count; i++) {
            setTimeout(() => {
                const particle = document.createElement('div');
                particle.className = 'celebration-particle';
                particle.style.left = Math.random() * window.innerWidth + 'px';
                particle.style.background = color;
                particle.style.animationDelay = Math.random() * 2 + 's';
                
                document.body.appendChild(particle);
                
                setTimeout(() => {
                    particle.remove();
                }, 3000);
            }, i * 50);
        }
    }

    playSuccessSound() {
        // Create audio context for success sound
        this.playTone(800, 200);
        setTimeout(() => this.playTone(1000, 200), 250);
    }

    playVictorySound() {
        // Create audio context for victory fanfare
        const notes = [523, 659, 784, 1047, 1319];
        notes.forEach((freq, index) => {
            setTimeout(() => this.playTone(freq, 300), index * 200);
        });
    }

    playTone(frequency, duration) {
        if (typeof AudioContext === 'undefined' && typeof webkitAudioContext === 'undefined') {
            return; // Audio not supported
        }

        const audioContext = new (AudioContext || webkitAudioContext)();
        const oscillator = audioContext.createOscillator();
        const gainNode = audioContext.createGain();

        oscillator.connect(gainNode);
        gainNode.connect(audioContext.destination);

        oscillator.frequency.value = frequency;
        oscillator.type = 'sine';

        gainNode.gain.setValueAtTime(0.3, audioContext.currentTime);
        gainNode.gain.exponentialRampToValueAtTime(0.01, audioContext.currentTime + duration / 1000);

        oscillator.start(audioContext.currentTime);
        oscillator.stop(audioContext.currentTime + duration / 1000);
    }

    startQuantumAnimations() {
        // Start particle animation
        this.animateQuantumParticles();
        
        // Start header glow animation
        this.animateHeaderGlow();
    }

    animateQuantumParticles() {
        // Enhanced particle system is handled by CSS
        // This could be extended for more complex particle effects
    }

    animateHeaderGlow() {
        // Enhanced glow effects are handled by CSS
        // This could be extended for more interactive effects
    }

    initializeQuantumEffects() {
        // Initialize advanced quantum effects
        this.setupMouseTracker();
        this.setupKeyboardShortcuts();
    }

    setupMouseTracker() {
        // Add quantum trail effect to mouse movement
        document.addEventListener('mousemove', (e) => {
            if (Math.random() < 0.1) { // 10% chance to create particle
                this.createMouseParticle(e.clientX, e.clientY);
            }
        });
    }

    createMouseParticle(x, y) {
        const particle = document.createElement('div');
        particle.style.position = 'fixed';
        particle.style.left = x + 'px';
        particle.style.top = y + 'px';
        particle.style.width = '4px';
        particle.style.height = '4px';
        particle.style.background = '#00d4ff';
        particle.style.borderRadius = '50%';
        particle.style.pointerEvents = 'none';
        particle.style.zIndex = '9999';
        particle.style.opacity = '0.8';
        particle.style.animation = 'fadeOut 1s ease-out forwards';

        document.body.appendChild(particle);

        setTimeout(() => {
            particle.remove();
        }, 1000);
    }

    setupKeyboardShortcuts() {
        document.addEventListener('keydown', (e) => {
            if (e.ctrlKey || e.metaKey) {
                switch (e.key) {
                    case '1':
                        e.preventDefault();
                        this.showSection('login');
                        break;
                    case '2':
                        e.preventDefault();
                        this.showSection('withdraw');
                        break;
                    case 'q':
                        e.preventDefault();
                        const quantumModeCheckbox = document.getElementById('quantumMode');
                        if (quantumModeCheckbox) {
                            quantumModeCheckbox.checked = !quantumModeCheckbox.checked;
                            this.quantumMode = quantumModeCheckbox.checked;
                            this.updateQuantumUI();
                        }
                        break;
                }
            }
        });
    }

    updateQuantumUI() {
        // Update UI based on quantum mode
        const body = document.body;
        if (this.quantumMode) {
            body.classList.add('quantum-mode');
        } else {
            body.classList.remove('quantum-mode');
        }
    }

    validateSeedPhrase(seedPhrase) {
        if (typeof seedPhrase !== 'string') {
            seedPhrase = seedPhrase.target?.value || '';
        }
        
        const words = seedPhrase.trim().split(/\s+/);
        return words.length === 24 && words.every(word => word.length >= 3);
    }

    validateSponsorPhrase(sponsorPhrase) {
        if (typeof sponsorPhrase !== 'string') {
            sponsorPhrase = sponsorPhrase.target?.value || '';
        }
        
        if (!sponsorPhrase.trim()) return true; // Optional field
        
        const words = sponsorPhrase.trim().split(/\s+/);
        return words.length === 24 && words.every(word => word.length >= 3);
    }

    validateAddress(address) {
        if (typeof address !== 'string') {
            address = address.target?.value || '';
        }
        
        // Basic Stellar address validation
        return address.length === 56 && address.startsWith('G');
    }

    showLoading(message) {
        // Implementation for loading state
        this.addLog('🔄 ' + message, 'info');
    }

    hideLoading() {
        // Implementation for hiding loading state
    }

    showSuccess(message) {
        this.addLog('✅ ' + message, 'success');
    }

    showError(message) {
        this.addLog('❌ ' + message, 'error');
    }
}

// Initialize the Quantum Bot Controller when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    window.quantumBot = new QuantumBotController();
});

// Add CSS for fade out animation
const style = document.createElement('style');
style.textContent = `
    @keyframes fadeOut {
        from { opacity: 0.8; transform: scale(1); }
        to { opacity: 0; transform: scale(0); }
    }
    
    .quantum-mode {
        filter: hue-rotate(45deg) brightness(1.1);
    }
`;
document.head.appendChild(style);