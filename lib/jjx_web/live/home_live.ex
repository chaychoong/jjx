defmodule JjxWeb.HomeLive do
  @moduledoc false
  use JjxWeb, :live_view

  alias Jj.FileSystem.Local
  alias Jj.Native

  @impl true
  def mount(_params, _session, socket) do
    {:ok,
     assign(socket,
       path: Local.get_home(),
       error: nil,
       workspace: nil,
       configs: [],
       show_settings_modal: false,
       revset: ""
     )}
  end

  @impl true
  def render(assigns) do
    ~H"""
    <Layouts.app flash={@flash}>
      <div class="mx-auto max-w-sm">
        <form phx-submit="validate_path" class="join">
          <label class="input join-item">
            <.icon name="hero-folder" /><input type="text" class="grow" name="path" value={@path} />
          </label>
          <button class="btn btn-neutral join-item" type="submit">Open</button>
        </form>
        <%= if @error do %>
          <p class="mt-2 text-sm text-error">{@error}</p>
        <% end %>
      </div>
      <%= if @workspace do %>
        <div class="mt-4">
          <div class="overflow-visible">
            <div class="w-full">
              <form phx-submit="change_revset" class="join w-full">
                <label class="input join-item w-full">
                  <.icon name="hero-magnifying-glass" />
                  <input type="text" class="grow w-full font-mono" name="revset" value={@revset} />
                </label>
              </form>
            </div>
            <table class="table">
              <thead>
                <tr>
                  <th>Commit ID</th>
                  <th>Message</th>
                  <th>Author</th>
                  <th>Timestamp</th>
                </tr>
              </thead>
              <tbody>
                <%= for commit <- @log do %>
                  <tr
                    class="cursor-pointer hover:bg-base-200"
                    phx-click="select_commit"
                    phx-value-change-id={commit.change_id}
                  >
                    <td class="font-mono">
                      <div class="flex">
                        <span class="font-bold text-orange-500">
                          {String.slice(commit.change_id, 0..(commit.change_id_short_len - 1))}
                        </span>
                        {String.slice(commit.change_id, commit.change_id_short_len..7)}
                      </div>
                      <div class="flex">
                        <span class="font-bold text-blue-500">
                          {String.slice(commit.commit_id, 0..(commit.commit_id_short_len - 1))}
                        </span>
                        {String.slice(commit.commit_id, commit.commit_id_short_len..7)}
                      </div>
                    </td>
                    <td>{commit.message_first_line}</td>
                    <td>
                      <div class="tooltip tooltip-right" data-tip={commit.author_email}>
                        {commit.author_name}
                      </div>
                    </td>
                    <td>
                      {commit.timestamp |> DateTime.from_unix!(:millisecond)}
                    </td>
                  </tr>
                <% end %>
              </tbody>
            </table>
            <div class="divider"></div>
            <button class="btn btn-outline btn-sm" phx-click="show_settings_modal">
              Show settings
            </button>
          </div>
        </div>
        <div class={"modal " <> if(@show_settings_modal, do: "modal-open", else: "") }>
          <div class="modal-box max-w-4xl w-full overflow-x-hidden">
            <h1 class="text-lg font-bold mb-4">Configs</h1>
            <div class="overflow-x-auto">
              <table class="table w-full max-w-full">
                <thead>
                  <tr>
                    <th>Config</th>
                    <th>Value</th>
                  </tr>
                </thead>
                <tbody>
                  <%= for {name, value} <- @configs do %>
                    <tr>
                      <td class="max-w-xs overflow-x-auto">
                        <pre class="truncate">{name}</pre>
                      </td>
                      <td class="max-w-md">
                        <pre class="overflow-x-auto">{value}</pre>
                      </td>
                    </tr>
                  <% end %>
                </tbody>
              </table>
            </div>
            <div class="modal-action">
              <button class="btn" phx-click="close_settings_modal">Close</button>
            </div>
          </div>
          <form phx-submit="close_settings_modal" class="modal-backdrop">
            <button>close</button>
          </form>
        </div>
      <% end %>
    </Layouts.app>
    """
  end

  @impl true
  def handle_event("validate_path", %{"path" => path}, socket) do
    if Local.validate_jj_repo(path) do
      workspace = Native.get_workspace(path)
      configs = Native.get_configs(path)

      revset =
        Enum.find_value(configs, "ancestors(@, 10)", fn
          {"revsets.log", value} -> String.trim(value, "\"")
          _ -> nil
        end)

      {:ok, log} = Native.log(workspace, revset)

      {:noreply,
       assign(socket,
         error: nil,
         path: path,
         workspace: workspace,
         configs: configs,
         log: log,
         show_settings_modal: false,
         revset: revset
       )}
    else
      {:noreply,
       assign(socket,
         error: ".jj not found in: #{path}",
         path: path,
         workspace: nil,
         configs: [],
         show_settings_modal: false,
         revset: ""
       )}
    end
  end

  @impl true
  def handle_event("show_settings_modal", _params, socket) do
    {:noreply, assign(socket, :show_settings_modal, true)}
  end

  @impl true
  def handle_event("close_settings_modal", _params, socket) do
    {:noreply, assign(socket, :show_settings_modal, false)}
  end

  @impl true
  def handle_event("change_revset", %{"revset" => revset}, socket) do
    case Native.log(socket.assigns.workspace, revset) do
      {:ok, log} ->
        {:noreply, assign(socket, revset: revset, log: log, error: nil)}

      {:error, error} ->
        {:noreply, assign(socket, revset: revset, error: error)}
    end
  end
end
