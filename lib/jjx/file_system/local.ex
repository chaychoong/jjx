defmodule Jj.FileSystem.Local do
  @moduledoc """
  Interact with the local file system.
  """

  def get_home do
    System.user_home!()
  end

  def validate_jj_repo(path) do
    File.exists?(path) && File.exists?(Path.join(path, ".jj"))
  end
end
