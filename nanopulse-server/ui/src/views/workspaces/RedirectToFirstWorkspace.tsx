import { useEffect } from "react";
import { useNavigate } from "react-router";
import { client } from "../../storage/client";

export function RedirectToFirstWorkspace() {
  const navigate = useNavigate();

  useEffect(() => {
    const fetch = async () => {
      const { data } = await client.GET("/api/workspaces", {
        params: {
          query: {
            limit: 1,
          },
        },
      });

      if (data && data.result.length > 0) {
        navigate(`/workspaces/${data.result[0].name}`);
      }
    };

    fetch();
  });

  return null;
}
