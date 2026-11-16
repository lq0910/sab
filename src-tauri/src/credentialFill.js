// 功能：仅填写当前精确来源内唯一普通登录表单，不提交或操作跨域 iframe；时间：2026-11-16；作者：lq。
(() => {
  try {
    if (window.top !== window || location.protocol !== 'https:' || location.origin !== __SAB_ORIGIN__) return 'origin-changed';
    const visible = input => !input.disabled && !input.readOnly && input.getClientRects().length > 0 && getComputedStyle(input).visibility !== 'hidden';
    const passwords = [...document.querySelectorAll('input[type="password"]')].filter(visible);
    if (passwords.length !== 1 || passwords[0].autocomplete === 'new-password') return 'no-login-form';
    const pass = passwords[0];
    if (pass.form && new URL(pass.form.action, location.href).origin !== location.origin) return 'unsupported-form';
    const scope = pass.form || document;
    // 功能：用户名候选限制在密码框所属表单，避免改写页面上的搜索和其他业务字段；时间：2026-11-16；作者：lq。
    const inputs = [...scope.querySelectorAll('input')].filter(input => visible(input) && /^(text|email|tel)$/.test(input.type));
    const user = inputs.find(input => input.autocomplete === 'username') || inputs.find(input => /user|email|login|account/i.test(`${input.name} ${input.id}`)) || (inputs.length === 1 ? inputs[0] : undefined);
    const data = __SAB_ACCOUNT__;
    // 功能：原生单行输入框会去掉换行，提前拒绝以免把已改变的凭据当作成功；时间：2026-11-16；作者：lq。
    if (/[\r\n]/.test(data.password) || /[\r\n]/.test(data.username)) return 'unsupported-form';
    if (data.password.length > 4096 || data.username.length > 1024) return 'unsupported-form';
    const setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value').set;
    const set = (input, value) => { setter.call(input, value); input.dispatchEvent(new Event('input', { bubbles: true })); input.dispatchEvent(new Event('change', { bubbles: true })); };
    if (user && data.username) set(user, data.username);
    set(pass, data.password);
    if (pass.value !== data.password || (user && data.username && user.value !== data.username)) return 'unsupported-form';
    return 'filled';
  } catch { return 'unsupported-form'; }
})()
