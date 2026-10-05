<script>
  import { onMount } from 'svelte';
  import { 
    UserCheck, Plus, Trash2, Edit2, ShieldAlert, 
    KeyRound, CheckCircle2, AlertCircle, X, Lock, Mail, User
  } from 'lucide-svelte';

  export let initialUsers = [];
  let users = initialUsers || [];
  let isLoading = false;
  let successNotice = '';
  let errorNotice = '';

  // Modal State
  let isModalOpen = false;
  let modalMode = 'create'; // 'create' | 'edit'
  let editId = null;
  let isSaving = false;

  // Form Fields
  let username = '';
  let email = '';
  let password = '';
  let confirmPassword = '';
  let role = 'admin';

  onMount(async () => {
    if (users.length === 0) {
      await reloadUsers();
    }
  });

  async function reloadUsers() {
    isLoading = true;
    try {
      const token = localStorage.getItem('admin_token');
      const res = await fetch('/api/v1/admin/users', {
        headers: {
          ...(token ? { Authorization: `Bearer ${token}` } : {})
        }
      });
      if (res.ok) {
        users = await res.json();
      }
    } catch (e) {
      console.error('Failed to reload admin users:', e);
    } finally {
      isLoading = false;
    }
  }

  function openCreateModal() {
    modalMode = 'create';
    editId = null;
    username = '';
    email = '';
    password = '';
    confirmPassword = '';
    role = 'admin';
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
    role = u.role || 'admin';
    errorNotice = '';
    isModalOpen = true;
  }

  async function handleSaveUser() {
    isSaving = true;
    errorNotice = '';

    const cleanUsername = username.trim();
    if (!cleanUsername) {
      errorNotice = 'Username cannot be empty.';
      isSaving = false;
      return;
    }

    if (modalMode === 'create') {
      if (!password || password.length < 8) {
        errorNotice = 'Password must be at least 8 characters long.';
        isSaving = false;
        return;
      }
      if (password !== confirmPassword) {
        errorNotice = 'Password and confirmation do not match.';
        isSaving = false;
        return;
      }
    } else {
      if (password && password.length < 8) {
        errorNotice = 'New password must be at least 8 characters long.';
        isSaving = false;
        return;
      }
      if (password && password !== confirmPassword) {
        errorNotice = 'New password and confirmation do not match.';
        isSaving = false;
        return;
      }
    }

    const token = localStorage.getItem('admin_token');
    const headers = {
      'Content-Type': 'application/json',
      ...(token ? { Authorization: `Bearer ${token}` } : {})
    };

    try {
      if (modalMode === 'create') {
        const res = await fetch('/api/v1/admin/users', {
          method: 'POST',
          headers,
          body: JSON.stringify({
            username: cleanUsername,
            email: email.trim() || null,
            password,
            role
          })
        });

        if (res.ok) {
          successNotice = `Admin user "${cleanUsername}" created successfully!`;
          isModalOpen = false;
          await reloadUsers();
          setTimeout(() => successNotice = '', 4000);
        } else {
          const err = await res.text();
          errorNotice = err || 'Failed to create admin user';
        }
      } else {
        const payload = {
          username: cleanUsername,
          email: email.trim() || null,
          role
        };
        if (password.trim() !== '') {
          payload.password = password;
        }

        const res = await fetch(`/api/v1/admin/users/${editId}`, {
          method: 'PUT',
          headers,
          body: JSON.stringify(payload)
        });

        if (res.ok) {
          successNotice = `Admin user "${cleanUsername}" updated successfully!`;
          isModalOpen = false;
          await reloadUsers();
          setTimeout(() => successNotice = '', 4000);
        } else {
          const err = await res.text();
          errorNotice = err || 'Failed to update admin user';
        }
      }
    } catch (e) {
      errorNotice = e.message || 'Error saving admin user';
    } finally {
      isSaving = false;
    }
  }

  async function handleDeleteUser(u) {
    if (users.length <= 1) {
      alert('Cannot delete the last admin user! At least one administrative user must remain.');
      return;
    }

    if (!confirm(`Are you sure you want to permanently delete admin user "${u.username}"?`)) return;

    const token = localStorage.getItem('admin_token');
    try {
      const res = await fetch(`/api/v1/admin/users/${u.id}`, {
        method: 'DELETE',
        headers: {
          ...(token ? { Authorization: `Bearer ${token}` } : {})
        }
      });
      if (res.ok) {
        users = users.filter(item => item.id !== u.id);
        successNotice = `Admin user "${u.username}" deleted successfully.`;
        setTimeout(() => successNotice = '', 3500);
      } else {
        const err = await res.text();
        alert('Could not delete user: ' + err);
      }
    } catch (e) {
      console.error('Failed to delete admin user:', e);
    }
  }
</script>

<div class="space-y-6">
  <!-- Top Banner & Action Button -->
  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
    <div>
      <h1 class="text-2xl font-black text-white tracking-tight flex items-center gap-2.5">
        <UserCheck size={24} class="text-orange-500" />
        Admin Users & Access Control
      </h1>
      <p class="text-xs text-slate-400 mt-1">
        Manage administrative users who have access to store configuration, orders, catalog, and inventory.
      </p>
    </div>

    <button
      type="button"
      on:click={openCreateModal}
      class="px-4 py-2.5 rounded-xl bg-orange-600 hover:bg-orange-500 text-white font-bold text-xs shadow-lg shadow-orange-600/30 flex items-center gap-2 transition-all self-start sm:self-auto"
    >
      <Plus size={16} />
      <span>Add New Admin User</span>
    </button>
  </div>

  {#if successNotice}
    <div class="p-3.5 rounded-xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 text-xs font-semibold flex items-center gap-2">
      <CheckCircle2 size={16} />
      <span>{successNotice}</span>
    </div>
  {/if}

  <!-- Admin Users Table -->
  <div class="rounded-2xl bg-slate-900 border border-slate-800 shadow-xl overflow-hidden">
    <div class="overflow-x-auto">
      <table class="w-full text-left text-xs">
        <thead class="bg-slate-950/60 border-b border-slate-800 text-slate-400 uppercase tracking-wider text-[11px]">
          <tr>
            <th class="py-3.5 px-4 font-bold">Username</th>
            <th class="py-3.5 px-4 font-bold">Email</th>
            <th class="py-3.5 px-4 font-bold">Role</th>
            <th class="py-3.5 px-4 font-bold">Credentials Status</th>
            <th class="py-3.5 px-4 font-bold">Created Date</th>
            <th class="py-3.5 px-4 font-bold text-right">Actions</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-slate-800/60">
          {#each users as u}
            <tr class="hover:bg-slate-800/40 transition-colors">
              <!-- Username -->
              <td class="py-3.5 px-4">
                <div class="flex items-center gap-2.5">
                  <div class="w-8 h-8 rounded-lg bg-slate-950 border border-slate-800 flex items-center justify-center font-bold text-orange-400 text-xs">
                    {u.username.substring(0, 2).toUpperCase()}
                  </div>
                  <span class="font-bold text-white text-sm">{u.username}</span>
                </div>
              </td>

              <!-- Email -->
              <td class="py-3.5 px-4 text-slate-300 font-mono text-[11px]">
                {u.email || '—'}
              </td>

              <!-- Role -->
              <td class="py-3.5 px-4">
                <span class="inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full text-[10px] font-bold bg-indigo-500/10 text-indigo-400 border border-indigo-500/20 uppercase font-mono">
                  {u.role || 'Admin'}
                </span>
              </td>

              <!-- Status -->
              <td class="py-3.5 px-4">
                {#if u.is_default}
                  <span class="inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full text-[10px] font-bold bg-amber-500/10 text-amber-400 border border-amber-500/20">
                    <ShieldAlert size={11} /> Default Credentials
                  </span>
                {:else}
                  <span class="inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full text-[10px] font-bold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                    <CheckCircle2 size={11} /> Customized
                  </span>
                {/if}
              </td>

              <!-- Created Date -->
              <td class="py-3.5 px-4 text-slate-400 text-[11px]">
                {u.created_at ? new Date(u.created_at).toLocaleDateString() : '—'}
              </td>

              <!-- Actions -->
              <td class="py-3.5 px-4 text-right">
                <div class="inline-flex items-center gap-1.5">
                  <button
                    type="button"
                    on:click={() => openEditModal(u)}
                    title="Edit User & Password"
                    class="p-1.5 rounded-lg text-slate-400 hover:text-white hover:bg-slate-800 transition-colors"
                  >
                    <Edit2 size={14} />
                  </button>
                  <button
                    type="button"
                    on:click={() => handleDeleteUser(u)}
                    title="Delete User"
                    disabled={users.length <= 1}
                    class="p-1.5 rounded-lg text-rose-400 hover:text-rose-300 hover:bg-rose-500/15 transition-colors disabled:opacity-30 disabled:cursor-not-allowed"
                  >
                    <Trash2 size={14} />
                  </button>
                </div>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  </div>
</div>

<!-- Modal: Create / Edit Admin User -->
{#if isModalOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-slate-950/80 backdrop-blur-sm animate-in fade-in duration-200">
    <div class="w-full max-w-md rounded-2xl bg-slate-900 border border-slate-800 shadow-2xl p-6 space-y-5">
      <div class="flex items-center justify-between border-b border-slate-800 pb-3">
        <h2 class="text-base font-bold text-white flex items-center gap-2">
          <UserCheck size={18} class="text-orange-400" />
          <span>{modalMode === 'create' ? 'Create New Admin User' : 'Edit Admin User'}</span>
        </h2>
        <button
          type="button"
          on:click={() => isModalOpen = false}
          class="p-1.5 rounded-lg text-slate-400 hover:text-white hover:bg-slate-800"
        >
          <X size={16} />
        </button>
      </div>

      {#if errorNotice}
        <div class="p-3 rounded-xl bg-rose-500/10 border border-rose-500/20 text-rose-400 text-xs font-semibold flex items-center gap-2">
          <AlertCircle size={15} />
          <span>{errorNotice}</span>
        </div>
      {/if}

      <div class="space-y-4 text-xs">
        <div>
          <label for="admin-user-username" class="block font-semibold text-slate-300 mb-1">Username</label>
          <div class="relative">
            <input
              id="admin-user-username"
              type="text"
              bind:value={username}
              required
              placeholder="e.g. store_manager"
              class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500 pl-9"
            />
            <User size={15} class="absolute left-3 top-1/2 -translate-y-1/2 text-slate-500" />
          </div>
        </div>

        <div>
          <label for="admin-user-email" class="block font-semibold text-slate-300 mb-1">Email Address (Optional)</label>
          <div class="relative">
            <input
              id="admin-user-email"
              type="email"
              bind:value={email}
              placeholder="e.g. manager@example.com"
              class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500 pl-9"
            />
            <Mail size={15} class="absolute left-3 top-1/2 -translate-y-1/2 text-slate-500" />
          </div>
        </div>

        <div>
          <label for="admin-user-password" class="block font-semibold text-slate-300 mb-1">
            {modalMode === 'create' ? 'Password (min. 8 characters)' : 'New Password (leave blank to keep current)'}
          </label>
          <div class="relative">
            <input
              id="admin-user-password"
              type="password"
              bind:value={password}
              placeholder={modalMode === 'create' ? '••••••••' : 'Leave empty to preserve existing password'}
              class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500 pl-9"
            />
            <Lock size={15} class="absolute left-3 top-1/2 -translate-y-1/2 text-slate-500" />
          </div>
        </div>

        {#if modalMode === 'create' || password.length > 0}
          <div>
            <label for="admin-user-confirm-password" class="block font-semibold text-slate-300 mb-1">Confirm Password</label>
            <div class="relative">
              <input
                id="admin-user-confirm-password"
                type="password"
                bind:value={confirmPassword}
                placeholder="Re-enter password"
                class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500 pl-9"
              />
              <KeyRound size={15} class="absolute left-3 top-1/2 -translate-y-1/2 text-slate-500" />
            </div>
          </div>
        {/if}
      </div>

      <div class="flex items-center justify-end gap-3 border-t border-slate-800 pt-4">
        <button
          type="button"
          on:click={() => isModalOpen = false}
          class="px-4 py-2 rounded-xl bg-slate-800 hover:bg-slate-700 text-slate-300 text-xs font-semibold"
        >
          Cancel
        </button>
        <button
          type="button"
          on:click={handleSaveUser}
          disabled={isSaving}
          class="px-4 py-2 rounded-xl bg-orange-600 hover:bg-orange-500 text-white text-xs font-bold shadow-lg shadow-orange-600/30 transition-all disabled:opacity-50"
        >
          {isSaving ? 'Saving...' : (modalMode === 'create' ? 'Create User' : 'Save Changes')}
        </button>
      </div>
    </div>
  </div>
{/if}
