/* Global Language Selector - Nethercore Docs */
(function() {
    const STORAGE_KEY = 'mdbook-tabs-lang';
    const LANGUAGES = ['Rust', 'C/C++', 'Zig'];
    const DEFAULT_LANG = 'Rust';

    function getStoredLang() {
        try {
            return localStorage.getItem(STORAGE_KEY) || DEFAULT_LANG;
        } catch (e) {
            return DEFAULT_LANG;
        }
    }

    function switchLanguage(lang) {
        // Click the first matching tab to trigger the existing tabs.js logic
        // This ensures localStorage is updated and all tabs sync
        const tab = document.querySelector(`.mdbook-tab[data-tabname="${lang}"]`);
        if (tab) {
            tab.click();
        }
    }

    function createDropdown() {
        const container = document.createElement('div');
        container.className = 'lang-selector';

        const label = document.createElement('label');
        label.htmlFor = 'global-lang-select';
        label.textContent = 'Language:';

        const select = document.createElement('select');
        select.id = 'global-lang-select';
        select.title = 'Select programming language for code examples';

        LANGUAGES.forEach(lang => {
            const option = document.createElement('option');
            option.value = lang;
            option.textContent = lang;
            select.appendChild(option);
        });

        // Set current value from storage
        select.value = getStoredLang();

        // Handle changes
        select.addEventListener('change', (e) => {
            switchLanguage(e.target.value);
        });

        container.appendChild(label);
        container.appendChild(select);
        return container;
    }

    function init() {
        // Inject dropdown into header's right-buttons area
        const rightButtons = document.querySelector('.right-buttons');
        if (rightButtons) {
            const dropdown = createDropdown();
            rightButtons.insertBefore(dropdown, rightButtons.firstChild);
        }

        // Sync dropdown with any storage changes (e.g., from other tabs)
        window.addEventListener('storage', (e) => {
            if (e.key === STORAGE_KEY) {
                const select = document.getElementById('global-lang-select');
                if (select && e.newValue) {
                    select.value = e.newValue;
                }
            }
        });
    }

    // Run on DOM ready
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }
})();
