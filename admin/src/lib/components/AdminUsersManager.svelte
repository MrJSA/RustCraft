<script>
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { UserCheck, Plus, Trash2, Edit2, ShieldAlert, KeyRound, CheckCircle2, AlertCircle, X, Lock, Mail, User, Check, Minus } from 'lucide-svelte';

  export let initialUsers = [];
  let users = initialUsers || [];
  let successNotice = '';
  let listError = '';

  // Admin sections — same keys as the backend permission system
  const SECTIONS = [
    { key: 'overview', label: 'Overview & Analytics' },
    { key: 'products', label: 'Products' },
    { key: 'orders', label: 'Orders & Slips' },
    { key: 'storefront', label: 'Storefront & Design' },
    { key: 'settings', label: 'Settings' }
  ];
  const ROLES = [
    { key: 'editor', label: 'Editor', hint: 'Products and orders' },
    { key: 'admin', label: 'Admin', hint: 'Everything except settings' },
    { key: 'superadmin', label: 'Superadmin', hint: 'Full access, cannot be restricted' }
  ];

  function roleDefaults(role) {
    return {
      overview: role !== 'editor',
      products: true,
      orders: true,
      storefront: role !== 'editor',
      settings: role === 'superadmin'
    };
  }

  $: me = $page.data.admin || {};
  $: iAmSuperadmin = me.role === 'superadmin';

  // Modal state
  let isModalOpen = false;
  let modalMode = 'create';
  let editId = null;
  let isSaving = false;
  let errorNotice = '';
  let username = '';
  let email = '';
  let password = '';
  let confirmPassword = '';
  let role = 'editor';
  let permissions = roleDefaults('editor');

  $: editingSelf = modalMode === 'edit' && (
    Boolean(me.id && editId === me.id) ||
    Boolean(me.username && users.find((u) => u.id === editId)?.username?.toLowerCase() === me.username?.toLowerCase())
  );

  onMount(() => {
    if (users.length === 0) reloadUsers();
  });

  async function readError(res, fallback) {
    const text = await res.text().catch(() => '');
    try {
      const body = JSON.parse(text);
      return body.error || body.message || fallback;
    } catch {
      return text || fallback;
    }
  }

  async function reloadUsers() {
    listError = '';
    const res = await fetch('/api/v1/admin/users').catch(() => null);
    if (res?.ok) users = await res.json();
    else listError = res ? await readError(res, 'Could not load users.') : 'Could not load users.';
  }

  function openCreateModal() {
    modalMode = 'create';
    editId = null;
    username = '';
    email = '';
    password = '';
    confirmPassword = '';
    role = 'editor';
    permissions = roleDefaults('editor');
    errorNotice = '';
    isModalOpen = true;
  }

  function openEditModal(u) {
    modalMode = 'edit';
    editId = u.id;
    username = u.username;
    email = u.email || '';
    password = '';
    confirmPassword = '';
    role = u.role || 'editor';
    permissions = { ...roleDefaults(role), ...(u.permissions || {}) };
    errorNotice = '';
    isModalOpen = true;
  }

  function changeRole(newRole) {
    role = newRole;
    permissions = roleDefaults(newRole);
  }

  async function handleSaveUser() {
    errorNotice = '';
    const cleanUsername = username.trim();
    if (!cleanUsername) {
      errorNotice = 'Username cannot be empty.';
      return;
    }
    if (modalMode === 'create' || password) {
      if (password.length < 12) {
        errorNotice = 'The password must be at least 12 characters long.';
        return;
      }
      if (password !== confirmPassword) {
        errorNotice = 'Password and confirmation do not match.';
        return;
      }
    }

    const payload = { username: cleanUsername, email: email.trim() };
    if (password) payload.password = password;
    if (!editingSelf) {
      payload.role = role;
      payload.permissions = permissions;
    }

    isSaving = true;
    try {
      const res = await fetch(modalMode === 'create' ? '/api/v1/admin/users' : `/api/v1/admin/users/${editId}`, {
        method: modalMode === 'create' ? 'POST' : 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload)
      });
      if (!res.ok) {
        errorNotice = await readError(res, 'Could not save the user.');
        return;
      }
      successNotice = `User "${cleanUsername}" ${modalMode === 'create' ? 'created' : 'updated'}.`;
      if (editingSelf && $page.data.admin) {
        $page.data.admin.username = cleanUsername;
        $page.data.admin.email = email.trim();
      }
      isModalOpen = false;
      await reloadUsers();
      setTimeout(() => (successNotice = ''), 4000);
    } catch (e) {
      errorNotice = e.message || 'Could not save the user.';
    } finally {
      isSaving = false;
    }
  }

  async function handleDeleteUser(u) {
    if (!confirm(`Permanently delete the admin user "${u.username}"?`)) return;
    const res = await fetch(`/api/v1/admin/users/${u.id}`, { method: 'DELETE' }).catch(() => null);
    if (res?.ok) {
      users = users.filter((item) => item.id !== u.id);
      successNotice = `User "${u.username}" deleted.`;
      setTimeout(() => (successNotice = ''), 3500);
    } else {
      alert('Could not delete user: ' + (res ? await readError(res, 'unknown error') : 'network error'));
    }
  }

  const inputClass =
    'w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500 pl-9';
</script>

<div class="space-y-6">
  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
    <div>
      <h1 class="text-2xl font-black text-white tracking-tight flex items-center gap-2.5">
        <UserCheck size={24} class="text-orange-500" />
        Admin Users & Access Control
      </h1>
      <p class="text-xs text-slate-400 mt-1">Create editors and admins and choose which admin sections each person may use.</p>
    </div>
    <button
      type="button"
      on:click={openCreateModal}
      class="px-4 py-2.5 rounded-xl bg-orange-600 hover:bg-orange-500 text-white font-bold text-xs shadow-lg shadow-orange-600/30 flex items-center gap-2 transition-all self-start sm:self-auto"
    >
      <Plus size={16} />
      <span>Add User (Editor / Admin)</span>
    </button>
  </div>

  {#if successNotice}
    <div class="p-3.5 rounded-xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 text-xs font-semibold flex items-center gap-2">
      <CheckCircle2 size={16} /> <span>{successNotice}</span>
    </div>
  {/if}
  {#if listError}
    <div class="p-3.5 rounded-xl bg-rose-500/10 border border-rose-500/20 text-rose-400 text-xs font-semibold flex items-center gap-2">
      <AlertCircle size={16} /> <span>{listError}</span>
    </div>
  {/if}

  <div class="rounded-2xl bg-slate-900 border border-slate-800 shadow-xl overflow-hidden">
    <div class="overflow-x-auto">
      <table class="w-full text-left text-xs">
        <thead class="bg-slate-950/60 border-b border-slate-800 text-slate-400 uppercase tracking-wider text-[10px]">
          <tr>
            <th class="py-3.5 px-4 font-bold">User</th>
            <th class="py-3.5 px-3 font-bold">Role</th>
            {#each SECTIONS as s}
              <th class="py-3.5 px-2 font-bold text-center">{s.label}</th>
            {/each}
            <th class="py-3.5 px-4 font-bold text-right">Actions</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-slate-800/60">
          {#each users as u (u.id)}
            <tr class="hover:bg-slate-800/40 transition-colors">
              <td class="py-3.5 px-4">
                <div class="font-bold text-white text-sm flex items-center gap-2">
                  {u.username}
                  {#if u.username === me.username}<span class="text-[9px] font-semibold text-slate-500 uppercase">(you)</span>{/if}
                </div>
                <div class="text-[11px] text-slate-500 font-mono">{u.email || '—'}</div>
                {#if u.is_default}
                  <span class="mt-1 inline-flex items-center gap-1 text-[10px] font-bold text-amber-400"><ShieldAlert size={11} /> initial password</span>
                {/if}
              </td>
              <td class="py-3.5 px-3">
                <span class="inline-flex px-2.5 py-0.5 rounded-full text-[10px] font-bold uppercase font-mono {u.role === 'superadmin' ? 'bg-orange-500/15 text-orange-400 border border-orange-500/30' : u.role === 'admin' ? 'bg-indigo-500/10 text-indigo-400 border border-indigo-500/20' : 'bg-slate-800 text-slate-300 border border-slate-700'}">
                  {u.role}
                </span>
              </td>
              {#each SECTIONS as s}
                <td class="py-3.5 px-2 text-center" title={s.label}>
                  {#if u.permissions?.[s.key]}
                    <Check size={15} class="inline text-emerald-400" />
                  {:else}
                    <Minus size={15} class="inline text-slate-600" />
                  {/if}
                </td>
              {/each}
              <td class="py-3.5 px-4 text-right whitespace-nowrap">
                <button
                  type="button"
                  on:click={() => openEditModal(u)}
                  disabled={u.role === 'superadmin' && !iAmSuperadmin}
                  title="Edit user & permissions"
                  class="p-1.5 rounded-lg text-slate-400 hover:text-white hover:bg-slate-800 transition-colors disabled:opacity-30"
                >
                  <Edit2 size={14} />
                </button>
                <button
                  type="button"
                  on:click={() => handleDeleteUser(u)}
                  disabled={u.username === me.username || (u.role === 'superadmin' && !iAmSuperadmin)}
                  title="Delete user"
                  class="p-1.5 rounded-lg text-rose-400 hover:text-rose-300 hover:bg-rose-500/15 transition-colors disabled:opacity-30 disabled:cursor-not-allowed"
                >
                  <Trash2 size={14} />
                </button>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  </div>
</div>

{#if isModalOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-slate-950/80 backdrop-blur-sm">
    <div class="w-full max-w-lg max-h-[90vh] overflow-y-auto rounded-2xl bg-slate-900 border border-slate-800 shadow-2xl p-6 space-y-5" role="dialog" aria-modal="true" aria-labelledby="user-modal-title">
      <div class="flex items-center justify-between border-b border-slate-800 pb-3">
        <h2 id="user-modal-title" class="text-base font-bold text-white flex items-center gap-2">
          <UserCheck size={18} class="text-orange-400" />
          <span>{modalMode === 'create' ? 'Add User' : `Edit ${username}`}</span>
        </h2>
        <button type="button" on:click={() => (isModalOpen = false)} class="p-1.5 rounded-lg text-slate-400 hover:text-white hover:bg-slate-800" aria-label="Close">
          <X size={16} />
        </button>
      </div>

      {#if errorNotice}
        <div class="p-3 rounded-xl bg-rose-500/10 border border-rose-500/20 text-rose-400 text-xs font-semibold flex items-center gap-2">
          <AlertCircle size={15} /> <span>{errorNotice}</span>
        </div>
      {/if}

      <div class="space-y-4 text-xs">
        <div>
          <label for="admin-user-username" class="block font-semibold text-slate-300 mb-1">Username</label>
          <div class="relative">
            <input id="admin-user-username" type="text" bind:value={username} autocomplete="off" placeholder="e.g. store_manager" class={inputClass} />
            <User size={15} class="absolute left-3 top-1/2 -translate-y-1/2 text-slate-500" />
          </div>
        </div>

        <div>
          <label for="admin-user-email" class="block font-semibold text-slate-300 mb-1">Email (optional)</label>
          <div class="relative">
            <input id="admin-user-email" type="email" bind:value={email} placeholder="e.g. manager@example.com" class={inputClass} />
            <Mail size={15} class="absolute left-3 top-1/2 -translate-y-1/2 text-slate-500" />
          </div>
        </div>

        <!-- Role -->
        <fieldset disabled={editingSelf}>
          <legend class="block font-semibold text-slate-300 mb-1.5">Role {#if editingSelf}<span class="text-slate-500 font-normal">(you cannot change your own role)</span>{/if}</legend>
          <div class="grid grid-cols-3 gap-2">
            {#each ROLES as r}
              <button
                type="button"
                on:click={() => changeRole(r.key)}
                disabled={editingSelf || (r.key === 'superadmin' && !iAmSuperadmin)}
                class="p-2.5 rounded-xl border text-left transition-all disabled:opacity-40 {role === r.key ? 'border-orange-500 bg-orange-600/10 ring-1 ring-orange-500' : 'border-slate-800 bg-slate-950 hover:border-slate-700'}"
              >
                <div class="font-bold text-white">{r.label}</div>
                <div class="text-[10px] text-slate-500">{r.hint}</div>
              </button>
            {/each}
          </div>
        </fieldset>

        <!-- Section permissions -->
        <fieldset disabled={editingSelf || role === 'superadmin'}>
          <legend class="block font-semibold text-slate-300 mb-1.5">
            Access to admin sections
            {#if role === 'superadmin'}<span class="text-slate-500 font-normal">(superadmins always have full access)</span>{/if}
          </legend>
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-2">
            {#each SECTIONS as s}
              <label class="p-2.5 rounded-xl bg-slate-950 border border-slate-800 flex items-center gap-2.5 cursor-pointer {editingSelf || role === 'superadmin' ? 'opacity-60 cursor-not-allowed' : 'hover:border-slate-700'}">
                <input type="checkbox" bind:checked={permissions[s.key]} class="w-4 h-4 accent-orange-600" />
                <span class="text-slate-200 font-semibold">{s.label}</span>
              </label>
            {/each}
          </div>
        </fieldset>

        <div>
          <label for="admin-user-password" class="block font-semibold text-slate-300 mb-1">
            {modalMode === 'create' ? 'Password (min. 12 characters)' : 'New password (leave empty to keep)'}
          </label>
          <div class="relative">
            <input id="admin-user-password" type="password" autocomplete="new-password" bind:value={password} class={inputClass} />
            <Lock size={15} class="absolute left-3 top-1/2 -translate-y-1/2 text-slate-500" />
          </div>
        </div>

        {#if modalMode === 'create' || password.length > 0}
          <div>
            <label for="admin-user-confirm-password" class="block font-semibold text-slate-300 mb-1">Confirm password</label>
            <div class="relative">
              <input id="admin-user-confirm-password" type="password" autocomplete="new-password" bind:value={confirmPassword} class={inputClass} />
              <KeyRound size={15} class="absolute left-3 top-1/2 -translate-y-1/2 text-slate-500" />
            </div>
          </div>
        {/if}
      </div>

      <div class="flex items-center justify-end gap-3 border-t border-slate-800 pt-4">
        <button type="button" on:click={() => (isModalOpen = false)} class="px-4 py-2 rounded-xl bg-slate-800 hover:bg-slate-700 text-slate-300 text-xs font-semibold">Cancel</button>
        <button
          type="button"
          on:click={handleSaveUser}
          disabled={isSaving}
          class="px-4 py-2 rounded-xl bg-orange-600 hover:bg-orange-500 text-white text-xs font-bold shadow-lg shadow-orange-600/30 transition-all disabled:opacity-50"
        >
          {isSaving ? 'Saving…' : modalMode === 'create' ? 'Create User' : 'Save Changes'}
        </button>
      </div>
    </div>
  </div>
{/if}
