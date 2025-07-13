defmodule JjxWeb.HomeLive do
  @moduledoc false
  use JjxWeb, :live_view

  alias Jj.FileSystem.Local
  alias Jj.Native

  @impl true
  def mount(_params, _session, socket) do
    {:ok, assign(socket, path: Local.get_home(), error: nil, workspace: nil)}
  end

  @impl true
  def render(assigns) do
    ~H"""
    <Layouts.app flash={@flash}>
      <div class="mx-auto max-w-sm">
        <form phx-change="update_path" class="join">
          <label class="input join-item">
            Path <input type="text" class="grow" name="path" value={@path} />
          </label>
        </form>
        <button class="btn btn-neutral join-item" phx-click="validate_path">Open</button>
        <%= if @error do %>
          <p class="mt-2 text-sm text-error">{@error}</p>
        <% end %>
      </div>
      <%= if @workspace do %>
        <div class="mt-4">
          <div class="overflow-visible">
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
          </div>
        </div>
      <% end %>
    </Layouts.app>
    """
  end

  @impl true
  def handle_event("update_path", %{"path" => path}, socket) do
    {:noreply, assign(socket, :path, path)}
  end

  @impl true
  def handle_event("validate_path", _params, socket) do
    if Local.validate_jj_repo(socket.assigns.path) do
      workspace = Native.get_workspace(socket.assigns.path)
      {:ok, log} = Native.simple_log(workspace)

      {:noreply,
       assign(socket,
         error: nil,
         workspace: workspace,
         log: log
       )}
    else
      {:noreply,
       assign(socket,
         error: ".jj not found in: #{socket.assigns.path}",
         workspace: nil
       )}
    end
  end
end
