defmodule Jj.Native.Commit do
  @moduledoc false
  defstruct id: nil,
            message_first_line: nil,
            author_name: nil,
            author_email: nil

  @type t :: %__MODULE__{
          id: String.t(),
          message_first_line: String.t(),
          author_name: String.t(),
          author_email: String.t()
        }
end

defmodule Jj.Native do
  @moduledoc false
  use Rustler, otp_app: :jjx, crate: "jj_nif"

  def get_workspace(_path), do: :erlang.nif_error(:nif_not_loaded)
  def simple_log(_workspace), do: :erlang.nif_error(:nif_not_loaded)
end
