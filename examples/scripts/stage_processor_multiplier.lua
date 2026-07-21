-- An example of simple Lua processor module
function process(data, headers)
  local num = tonumber(data)
  if num then
   sendMessage(tostring(num * 10), headers)
  else
   sendMessage(data, headers)
  end
end