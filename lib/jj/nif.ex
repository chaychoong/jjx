defmodule Jj.Nif do
  @moduledoc false
  use Rustler, otp_app: :jjx, crate: "jj_nif"

  # When your NIF is loaded, it will override this function.
  def add(_a, _b), do: :erlang.nif_error(:nif_not_loaded)
end
