defmodule Jj.Native.Commit do
  @moduledoc false
  defstruct change_id: nil,
            change_id_short_len: nil,
            commit_id: nil,
            commit_id_short_len: nil,
            message_first_line: nil,
            author_name: nil,
            author_email: nil,
            timestamp: nil

  @type t :: %__MODULE__{
          change_id: String.t(),
          change_id_short_len: integer(),
          commit_id: String.t(),
          commit_id_short_len: integer(),
          message_first_line: String.t(),
          author_name: String.t(),
          author_email: String.t(),
          timestamp: integer()
        }
end

defmodule Jj.Native do
  @moduledoc false
  use Rustler, otp_app: :jjx, crate: "jj_nif"

  def get_workspace(_path), do: :erlang.nif_error(:nif_not_loaded)
  def simple_log(_workspace), do: :erlang.nif_error(:nif_not_loaded)
end
