defmodule JjxWeb.PageController do
  use JjxWeb, :controller

  def home(conn, _params) do
    render(conn, :home)
  end
end
