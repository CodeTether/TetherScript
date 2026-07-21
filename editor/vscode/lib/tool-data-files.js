const files = {
  fs_copy: ['fs_copy(from, to)', 'Copy a filesystem entry.'],
  fs_exists: ['fs_exists(path)', 'Return whether a path exists.'],
  fs_list: ['fs_list(path)', 'List directory entries.'],
  fs_mkdir: ['fs_mkdir(path)', 'Create a directory.'],
  fs_read: ['fs_read(path)', 'Read a UTF-8 file.'],
  fs_remove: ['fs_remove(path)', 'Remove a filesystem entry.'],
  fs_rename: ['fs_rename(from, to)', 'Rename a filesystem entry.'],
  fs_stat: ['fs_stat(path)', 'Return filesystem metadata.'],
  fs_write: ['fs_write(path, body)', 'Write a UTF-8 file.'],
  path_basename: ['path_basename(path)', 'Return the final path component.'],
  path_dirname: ['path_dirname(path)', 'Return the parent directory.'],
  path_extname: ['path_extname(path)', 'Return the filename extension.'],
  path_join: ['path_join(parts)', 'Join a list of path components.'],
  path_normalize: ['path_normalize(path)', 'Normalize path components.'],
  path_resolve: ['path_resolve(path)', 'Resolve a path from the working directory.'],
  path_sep: ['path_sep()', 'Return the platform path separator.'],
};

module.exports = { files };
