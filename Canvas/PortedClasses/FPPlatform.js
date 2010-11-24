// Copyright 2010 Filip Kunc. All rights reserved.

var platformImage = new Image();
platformImage.src = "Images/plos_marble.png";

function FPPlatform(x, y, widthSegments, heightSegments)
{
    this.x = x;
    this.y = y;
    this.widthSegments = widthSegments;
    this.heightSegments = heightSegments;
    this.isVisible = true;

    this.isPlatform = function()
    {
        return true;
    }
    
    this.isMovable = function()
    {
        return false;
    }
    
    this.rect = function()
    {
        return new FPRect(this.x, this.y, this.widthSegments * 32.0, this.heightSegments * 32.0);
    }
    
    this.move = function(offsetX, offsetY)
    {
        this.x += offsetX;
        this.y += offsetY;
    }
    
    this.update = function(game)
    {
        
    }
    
    this.draw = function(context)
    {
        for (iy = 0; iy < this.heightSegments; iy++)
        {
            for (ix = 0; ix < this.widthSegments; ix++)
            {
                context.drawImage(platformImage, this.x + ix * 32.0, this.y + iy * 32.0);
            }
        }
    }
}