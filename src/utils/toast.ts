export function showToast(msg: string, type: 'info' | 'error' | 'success' | 'warning' = 'info', duration = 3000) {
    const toastContainer = document.createElement('div');
    toastContainer.className = 'toast toast-top toast-center z-[9999]';
    
    const alert = document.createElement('div');
    alert.className = `alert shadow-lg transition-opacity duration-300 opacity-0 bg-base-100/90 backdrop-blur border-none pointer-events-none flex items-center pr-6`;
    
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
    
    alert.appendChild(iconWrapper.firstElementChild as Node);
    alert.appendChild(text);
    toastContainer.appendChild(alert);
    document.body.appendChild(toastContainer);
    
    // trigger animation
    requestAnimationFrame(() => {
        alert.classList.remove('opacity-0');
        alert.classList.add('opacity-100');
    });

    setTimeout(() => {
        alert.classList.remove('opacity-100');
        alert.classList.add('opacity-0');
        setTimeout(() => {
            if (document.body.contains(toastContainer)) {
                document.body.removeChild(toastContainer);
            }
        }, 300); // Wait for transition
    }, duration);
}
