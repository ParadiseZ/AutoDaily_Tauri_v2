let toastContainer: HTMLElement | null = null;

function getToastContainer() {
    if (!toastContainer || !document.body.contains(toastContainer)) {
        toastContainer = document.createElement('div');
        toastContainer.className = 'toast toast-top toast-center z-[9999] pointer-events-none flex flex-col gap-3 mt-2';
        document.body.appendChild(toastContainer);
    }
    return toastContainer;
}

export function showToast(msg: string, type: 'info' | 'error' | 'success' | 'warning' = 'info', duration = 3000) {
    const container = getToastContainer();
    
    const alert = document.createElement('div');
    alert.className = `alert shadow-lg transition-all duration-300 opacity-0 -translate-y-6 bg-base-100/90 backdrop-blur border-none pointer-events-auto flex items-center pr-6 relative overflow-hidden`;
    
    let svgInner = '';

    // Add appropriate text color and svg icon content
    if (type === 'error') {
        alert.classList.add('text-error');
        svgInner = '<circle cx="12" cy="12" r="12" fill="currentColor"/><path d="M8 8l8 8M16 8l-8 8" stroke="white" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" fill="none"/>';
    } else if (type === 'success') {
        alert.classList.add('text-success');
        svgInner = '<circle cx="12" cy="12" r="12" fill="currentColor"/><path d="M7 13l3 3 7-7" stroke="white" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" fill="none"/>';
    } else if (type === 'warning') {
        alert.classList.add('text-warning');
        svgInner = '<circle cx="12" cy="12" r="12" fill="currentColor"/><path d="M12 7v5m0 4v.01" stroke="white" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" fill="none"/>';
    } else {
        alert.classList.add('text-info');
        svgInner = '<circle cx="12" cy="12" r="12" fill="currentColor"/><text x="12" y="16.5" font-style="italic" font-weight="bold" font-family="serif" font-size="14" fill="white" text-anchor="middle">i</text>';
    }

    const iconWrapper = document.createElement('div');
    iconWrapper.innerHTML = `<svg width="20" height="20" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" class="shrink-0">${svgInner}</svg>`;

    const text = document.createElement('span');
    text.className = 'font-medium';
    text.textContent = msg;
    
    // Create progress bar
    const progress = document.createElement('div');
    progress.className = 'absolute bottom-0 left-0 h-[3px] bg-current opacity-40';
    progress.style.width = '100%';
    progress.style.transition = `width ${duration}ms linear`;

    alert.appendChild(iconWrapper.firstElementChild as Node);
    alert.appendChild(text);
    alert.appendChild(progress);
    
    container.appendChild(alert);
    
    // trigger animation
    requestAnimationFrame(() => {
        requestAnimationFrame(() => {
            alert.classList.remove('opacity-0', '-translate-y-6');
            alert.classList.add('opacity-100', 'translate-y-0');
            progress.style.width = '0%';
        });
    });

    let isRemoving = false;
    const removeToast = () => {
        if (isRemoving) return;
        isRemoving = true;
        
        alert.classList.remove('opacity-100', 'translate-y-0');
        alert.classList.add('opacity-0', '-translate-y-6');
        
        setTimeout(() => {
            if (alert.parentNode) {
                alert.parentNode.removeChild(alert);
            }
            if (container.childNodes.length === 0 && container.parentNode) {
                container.parentNode.removeChild(container);
                toastContainer = null;
            }
        }, 300); // Wait for transition
    };

    setTimeout(removeToast, duration);
}
