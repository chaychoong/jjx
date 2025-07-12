defmodule Jjx.Repo do
  use Ecto.Repo,
    otp_app: :jjx,
    adapter: Ecto.Adapters.SQLite3
end
